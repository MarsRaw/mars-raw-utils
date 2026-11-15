#!/usr/bin/env python3
"""
Assemble cylindrically-projected Mars Raw Utils frames into a single mosaic.

Note: Claude wrote this from a set of sample input files and I have not review the code. 
Seems to work well on MSL Sol 5073 test files, but that's about the extent of testing.

Each input is a 16-bit TIFF (e.g. NLB_..._-rjcal-cyl.tif or ..-rjcal-pol.tif)
with a sidecar JSON (..-cyl-metadata.json / ..-pol-metadata.json) whose
"mosaic.projection.parameters" block gives the frame's placement:

  Cylindrical family (equirectangular, mercator, equal-area, perspective)
              : left_lon, top_lat, degrees_per_pixel         (wraps at 0/360)
                In every variant x is linear in longitude. top_lat / bottom_lat
                are the *projected* y values in degrees (y = f(lat) * 180/pi with
                f = lat, asinh(tan lat), sin lat or tan lat respectively), so
                frames of one variant share a single y axis.
  Polar family (polar-rectilinear, -stereographic, -equal-area, -orthographic)
              : max_radius, max_edge_length (eye_height is carried through)
                Pixel (0,0) is plane (-max_radius, -max_radius), pixels are
                2*max_radius/max_edge_length plane units, +x right, +y down.

All inputs must use one projection variant; a mix (cyl + pol, or e.g. mercator +
equal-area, or polar-rectilinear + polar-stereographic) is an error. The variant
is read from the "-p" option in the last "mru project" history entry, because the
"projection" name string in the metadata is not reliable.
The older flat metadata layout ("projection": {left_lon, ...}) is still read.

Pipeline
  1. Read frames + metadata, work out each frame's pixel offset on a shared canvas
     (handles longitude wrap-around at 0/360).
  2. Build a clean validity mask per frame (drops the black margin, fills holes,
     erodes a few pixels to lose the dark interpolated edge).
  3. Exposure compensation: solve one gain per frame by least squares so overlap
     regions agree (with a prior pulling gains toward 1.0).
  4. Seams: each canvas pixel is owned by the frame in which it is furthest from
     that frame's edge.
  5. Multi-band (Laplacian pyramid) blending across the seams.
  6. Write a 16-bit RGBA TIFF (alpha = coverage), a metadata JSON describing the
     mosaic's projection, and optionally an 8-bit preview PNG.

Requires: numpy, opencv-python

Usage
  python mosaic.py /path/to/dir --type cyl -o mosaic.tif --preview
  python mosaic.py *-pol.tif -o mosaic.tif
"""

import argparse
import glob
import json
import math
import os
import re
import sys
from dataclasses import dataclass

import cv2
import numpy as np


def log(*a):
    print(*a, file=sys.stderr, flush=True)


# --------------------------------------------------------------------------- #
# Projection metadata
# --------------------------------------------------------------------------- #
@dataclass
class Geo:
    """Placement of a frame in a flat 'plane' where +x is right and +y is down.
    Cylindrical: x = longitude (deg), y = -latitude (deg).
    Polar:       x, y in the projection's plane units."""
    kind: str          # "cyl" or "pol"
    name: str          # projection variant from the command history, if known
    ox: float          # plane x of the frame's pixel (0, 0)
    oy: float          # plane y of the frame's pixel (0, 0)
    psize: float       # plane units per pixel
    edge: int = 0      # polar: declared max_edge_length
    eye_height: float = None   # polar: carried through to the output metadata


# Older runs used -p cylindrical; its y axis is linear in latitude, i.e. the same
# as equirectangular, so the two may be combined.
VARIANT_ALIASES = {"cylindrical": "equirectangular"}


def read_geo(meta, path):
    proj = (meta.get("mosaic") or {}).get("projection") or meta.get("projection")
    if proj is None:
        raise ValueError(f"{path}: metadata has no projection block")
    params = proj.get("parameters")
    if params is None:                       # older flat layout
        cyl, pol = (proj, None) if "left_lon" in proj else (None, None)
    else:                                    # trust the parameter block, not the
        cyl, pol = params.get("Cylindrical"), params.get("Polar")   # name string
    if cyl is not None and pol is None:
        geo = Geo("cyl", None, cyl["left_lon"], -cyl["top_lat"], cyl["degrees_per_pixel"])
    elif pol is not None and cyl is None:
        r, e = pol["max_radius"], int(pol["max_edge_length"])
        geo = Geo("pol", None, -r, -r, 2.0 * r / e, e, pol.get("eye_height"))
    else:
        raise ValueError(f"{path}: unrecognised projection parameters "
                         f"({list(params) if params else list(proj)})")
    for entry in reversed(meta.get("processing", {}).get("history", [])):
        m = re.search(r"mru project\b.*?(?:\s-p|\s--projection)[ =]+([\w-]+)", entry)
        if m:
            geo.name = VARIANT_ALIASES.get(m.group(1).lower(), m.group(1).lower())
            break
    return geo


def check_single_type(paths, geos):
    kinds = {}
    for p, g in zip(paths, geos):
        kinds.setdefault(g.kind, []).append(os.path.basename(p))
    if len(kinds) > 1:
        desc = "; ".join(f"{len(v)} {k} (e.g. {v[0]})" for k, v in kinds.items())
        sys.exit(f"ERROR: inputs mix projection types: {desc}.\n"
                 "Use one projection type at a time (try --type cyl or --type pol).")
    names = {}
    for p, g in zip(paths, geos):
        if g.name:
            names.setdefault(g.name, []).append(os.path.basename(p))
    if len(names) > 1:
        desc = "; ".join(f"{len(v)} x {k} (e.g. {v[0]})" for k, v in names.items())
        sys.exit(f"ERROR: inputs mix projection variants: {desc}.")
    return next(iter(kinds))


# --------------------------------------------------------------------------- #
# Loading
# --------------------------------------------------------------------------- #
@dataclass
class Tile:
    path: str
    name: str
    geo: Geo = None
    img: np.ndarray = None        # float32, HxW (gray) or HxWx3, range 0..1
    mask: np.ndarray = None       # bool HxW
    cx: int = 0                   # crop offset inside the original frame
    cy: int = 0
    x0: int = 0                   # canvas offset (pixels) of img's top-left
    y0: int = 0
    gain: float = 1.0
    gray: bool = True

    @property
    def h(self):
        return self.img.shape[0]

    @property
    def w(self):
        return self.img.shape[1]


def find_inputs(items, kind=None):
    suffixes = [kind] if kind else ["cyl", "pol"]
    paths = []
    for it in items:
        if os.path.isdir(it):
            for sfx in suffixes:
                paths += sorted(glob.glob(os.path.join(it, f"*-{sfx}.tif")))
        else:
            hits = sorted(glob.glob(it))
            paths += hits if hits else [it]
    seen, out = set(), []
    for p in paths:
        if p not in seen and p.lower().endswith((".tif", ".tiff")):
            seen.add(p)
            out.append(p)
    return out


def metadata_path(tif):
    stem = re.sub(r"\.tiff?$", "", tif, flags=re.I)
    for cand in (stem + "-metadata.json", stem + ".json"):
        if os.path.exists(cand):
            return cand
    raise FileNotFoundError(f"No metadata JSON found next to {tif}")


def clean_mask(valid, erode_px):
    """Close pinholes, keep the biggest blob, fill interior holes, erode edge."""
    m = valid.astype(np.uint8)
    m = cv2.morphologyEx(m, cv2.MORPH_CLOSE, np.ones((3, 3), np.uint8))
    n, lab, stats, _ = cv2.connectedComponentsWithStats(m, connectivity=4)
    if n > 1:
        biggest = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA]))
        m = (lab == biggest).astype(np.uint8)
    mp = np.pad(m, 1)
    n, lab = cv2.connectedComponents((mp == 0).astype(np.uint8), connectivity=4)
    m = (lab[1:-1, 1:-1] != lab[0, 0]).astype(np.uint8)
    if erode_px > 0:
        k = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * erode_px + 1,) * 2)
        m = cv2.erode(m, k)
    return m.astype(bool)


def load_tile(path, geo, target_psize, erode_px):
    raw = cv2.imread(path, cv2.IMREAD_UNCHANGED)
    if raw is None:
        raise IOError(f"Could not read {path}")
    if raw.ndim == 2:
        raw = raw[..., None]
    if geo.kind == "pol" and max(raw.shape[:2]) != geo.edge:
        log(f"  warning: {os.path.basename(path)} is {raw.shape[1]}x{raw.shape[0]} "
            f"but metadata says max_edge_length={geo.edge}")
    scale = 65535.0 if raw.dtype == np.uint16 else 255.0

    color = raw[..., :3]
    valid = color.max(axis=2) > 0
    if raw.shape[2] == 4 and raw[..., 3].min() != raw[..., 3].max():
        valid &= raw[..., 3] > 0            # honour a real alpha channel

    gray = color.shape[2] < 3 or (
        np.array_equal(color[..., 0], color[..., 1])
        and np.array_equal(color[..., 0], color[..., 2])
    )
    img = (color[..., 0] if gray else color).astype(np.float32) / scale

    # Resample if this frame's scale differs from the mosaic scale.
    s = geo.psize / target_psize
    if abs(s - 1.0) > 1e-6:
        nw, nh = max(1, round(img.shape[1] * s)), max(1, round(img.shape[0] * s))
        img = cv2.resize(img, (nw, nh), interpolation=cv2.INTER_LINEAR)
        valid = cv2.resize(valid.astype(np.uint8), (nw, nh),
                           interpolation=cv2.INTER_NEAREST).astype(bool)

    mask = clean_mask(valid, erode_px)
    if not mask.any():
        return None

    ys, xs = np.where(mask)
    cy, cx = int(ys.min()), int(xs.min())
    sl = (slice(cy, int(ys.max()) + 1), slice(cx, int(xs.max()) + 1))
    t = Tile(path=path, name=os.path.basename(path), geo=geo, gray=gray, cx=cx, cy=cy)
    t.img = np.ascontiguousarray(img[sl])
    t.mask = np.ascontiguousarray(mask[sl])
    return t


# --------------------------------------------------------------------------- #
# Placement on the shared canvas
# --------------------------------------------------------------------------- #
def find_lon_origin(tiles, psize):
    """Pick a longitude origin in the biggest uncovered gap, so frames that
    straddle 0/360 (or -180/180) still land contiguously."""
    binw = 0.05
    nb = int(round(360 / binw))
    cov = np.zeros(nb, bool)
    for t in tiles:
        left = t.geo.ox + t.cx * psize
        a = int(math.floor(left / binw))
        b = int(math.ceil((left + t.w * psize) / binw))
        for i in range(a, b + 1):
            cov[i % nb] = True
    if cov.all():
        return min(t.geo.ox + t.cx * psize for t in tiles)
    start = int(np.argmax(cov))
    best_len, best_end, run = 0, start, 0
    for k in range(nb):
        i = (start + k) % nb
        if not cov[i]:
            run += 1
        else:
            if run > best_len:
                best_len, best_end = run, i
            run = 0
    if run > best_len:
        best_end = start
    return best_end * binw


def place_tiles(tiles, kind, psize):
    """Assign canvas offsets. Returns (x_ref, y_ref, W, H) where x_ref / y_ref are
    the plane coordinates of the canvas' top-left corner."""
    if kind == "cyl":
        origin = find_lon_origin(tiles, psize)
        # wrap using the *cropped* left edge, which is what find_lon_origin binned
        xs = [((t.geo.ox + t.cx * psize - origin) % 360.0) / psize for t in tiles]
    else:
        origin = min(t.geo.ox for t in tiles)
        xs = [(t.geo.ox - origin) / psize + t.cx for t in tiles]
    oy_min = min(t.geo.oy for t in tiles)
    ys = [(t.geo.oy - oy_min) / psize + t.cy for t in tiles]
    for t, x, y in zip(tiles, xs, ys):
        t.x0, t.y0 = int(round(x)), int(round(y))
    minx = min(t.x0 for t in tiles)
    miny = min(t.y0 for t in tiles)
    for t in tiles:
        t.x0 -= minx
        t.y0 -= miny
    W = max(t.x0 + t.w for t in tiles)
    H = max(t.y0 + t.h for t in tiles)
    return origin + minx * psize, oy_min + miny * psize, W, H


# --------------------------------------------------------------------------- #
# Exposure compensation
# --------------------------------------------------------------------------- #
def luminance(a):
    return a if a.ndim == 2 else a.mean(axis=2)


def solve_gains(tiles, sigma_n=10 / 255.0, sigma_g=0.1, min_overlap=500):
    """
    Least squares for per-tile gains g_i:
        minimise  sum_ij  N_ij (g_i*mean_i|ij - g_j*mean_j|ij)^2 / sigma_n^2
                + sum_i   N_i  (1 - g_i)^2 / sigma_g^2
    """
    n = len(tiles)
    A, b = [], []
    tot = np.zeros(n)
    for i in range(n):
        ti = tiles[i]
        for j in range(i + 1, n):
            tj = tiles[j]
            xa, xb = max(ti.x0, tj.x0), min(ti.x0 + ti.w, tj.x0 + tj.w)
            ya, yb = max(ti.y0, tj.y0), min(ti.y0 + ti.h, tj.y0 + tj.h)
            if xa >= xb or ya >= yb:
                continue
            si = (slice(ya - ti.y0, yb - ti.y0), slice(xa - ti.x0, xb - ti.x0))
            sj = (slice(ya - tj.y0, yb - tj.y0), slice(xa - tj.x0, xb - tj.x0))
            li, lj = luminance(ti.img[si]), luminance(tj.img[sj])
            ok = ti.mask[si] & tj.mask[sj]
            ok &= (li > 0.005) & (li < 0.98) & (lj > 0.005) & (lj < 0.98)
            cnt = int(ok.sum())
            if cnt < min_overlap:
                continue
            mi, mj = float(li[ok].mean()), float(lj[ok].mean())
            w = math.sqrt(cnt) / sigma_n
            row = np.zeros(n)
            row[i], row[j] = mi * w, -mj * w
            A.append(row)
            b.append(0.0)
            tot[i] += cnt
            tot[j] += cnt
    for i in range(n):
        w = math.sqrt(max(tot[i], 1.0)) / sigma_g
        row = np.zeros(n)
        row[i] = w
        A.append(row)
        b.append(w)
    g, *_ = np.linalg.lstsq(np.array(A), np.array(b), rcond=None)
    return np.clip(g, 0.25, 4.0)


# --------------------------------------------------------------------------- #
# Seams + multi-band blending
# --------------------------------------------------------------------------- #
def build_label_map(tiles, H, W):
    """Each canvas pixel is owned by the tile where it is furthest from the edge."""
    best = np.full((H, W), -1.0, np.float32)
    label = np.full((H, W), -1, np.int16)
    for idx, t in enumerate(tiles):
        padded = np.pad(t.mask.astype(np.uint8), 1)
        d = cv2.distanceTransform(padded, cv2.DIST_L2, 3)[1:-1, 1:-1]
        sl = (slice(t.y0, t.y0 + t.h), slice(t.x0, t.x0 + t.w))
        bv, lv = best[sl], label[sl]
        upd = (d > bv) & t.mask
        bv[upd] = d[upd]
        lv[upd] = idx
    return label


def extend_valid(img, mask, reach, radius=8):
    """Push valid colours outward into the invalid area so the black margin never
    leaks into the Gaussian pyramids."""
    img = img.copy()
    m = mask.astype(np.float32)
    k = (2 * radius + 1, 2 * radius + 1)
    for _ in range(int(math.ceil(reach / radius)) + 1):
        inv = m < 0.5
        if not inv.any():
            break
        den = cv2.blur(m, k)
        if img.ndim == 2:
            num = cv2.blur(img * m, k)
        else:
            num = cv2.blur(img * m[..., None], k)
        fill = inv & (den > 0.02)
        if not fill.any():
            break
        img[fill] = num[fill] / (den[fill][..., None] if img.ndim == 3 else den[fill])
        m[fill] = 1.0
    return img


def multiband_blend(tiles, label, H, W, levels):
    align = 1 << levels
    Hc, Wc = int(math.ceil(H / align)) * align, int(math.ceil(W / align)) * align
    gray = tiles[0].img.ndim == 2
    shp = lambda k: ((Hc >> k, Wc >> k) if gray else (Hc >> k, Wc >> k, 3))
    num = [np.zeros(shp(k), np.float32) for k in range(levels + 1)]
    den = [np.zeros((Hc >> k, Wc >> k), np.float32) for k in range(levels + 1)]
    lab_pad = np.full((Hc, Wc), -1, np.int16)
    lab_pad[:H, :W] = label

    for idx, t in enumerate(tiles):
        log(f"  blending {idx + 1}/{len(tiles)}: {t.name}")
        ax0, ay0 = (t.x0 // align) * align, (t.y0 // align) * align
        ax1 = int(math.ceil((t.x0 + t.w) / align)) * align
        ay1 = int(math.ceil((t.y0 + t.h) / align)) * align
        sh = (ay1 - ay0, ax1 - ax0)
        sub = np.zeros(sh if gray else sh + (3,), np.float32)
        msk = np.zeros(sh, bool)
        ys, xs = slice(t.y0 - ay0, t.y0 - ay0 + t.h), slice(t.x0 - ax0, t.x0 - ax0 + t.w)
        sub[ys, xs] = t.img * t.gain
        msk[ys, xs] = t.mask
        sub = extend_valid(sub, msk, reach=2 * align)
        wgt = (lab_pad[ay0:ay1, ax0:ax1] == idx).astype(np.float32)

        gi, gw = [sub], [wgt]
        for _ in range(levels):
            gi.append(cv2.pyrDown(gi[-1]))
            gw.append(cv2.pyrDown(gw[-1]))
        for k in range(levels + 1):
            if k < levels:
                up = cv2.pyrUp(gi[k + 1], dstsize=(gi[k].shape[1], gi[k].shape[0]))
                lap = gi[k] - up
            else:
                lap = gi[k]
            y_a, x_a = ay0 >> k, ax0 >> k
            y_b, x_b = y_a + lap.shape[0], x_a + lap.shape[1]
            wk = gw[k]
            num[k][y_a:y_b, x_a:x_b] += lap * (wk if gray else wk[..., None])
            den[k][y_a:y_b, x_a:x_b] += wk

    bands = []
    for k in range(levels + 1):
        d = den[k] if gray else den[k][..., None]
        bands.append(np.where(d > 1e-6, num[k] / np.maximum(d, 1e-6), 0).astype(np.float32))
    out = bands[levels]
    for k in range(levels - 1, -1, -1):
        out = cv2.pyrUp(out, dstsize=(bands[k].shape[1], bands[k].shape[0])) + bands[k]
    return out[:H, :W]


# --------------------------------------------------------------------------- #
# Main
# --------------------------------------------------------------------------- #
def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("inputs", nargs="+",
                    help="Directory of *-cyl.tif / *-pol.tif files, or files / globs")
    ap.add_argument("-o", "--output", default="mosaic.tif")
    ap.add_argument("--type", choices=["cyl", "pol"],
                    help="When an input is a directory, only pick up this projection type")
    ap.add_argument("--levels", type=int, default=5,
                    help="Multi-band pyramid levels (0 = hard seams). Default 5")
    ap.add_argument("--erode", type=int, default=3,
                    help="Pixels trimmed from each frame's valid edge. Default 3")
    ap.add_argument("--no-gain", action="store_true", help="Skip exposure compensation")
    ap.add_argument("--preview", action="store_true", help="Also write an 8-bit PNG preview")
    args = ap.parse_args()

    paths = find_inputs(args.inputs, args.type)
    if not paths:
        sys.exit("No input TIFFs found")
    log(f"{len(paths)} input images")

    geos = []
    for p in paths:
        with open(metadata_path(p)) as f:
            geos.append(read_geo(json.load(f), p))
    kind = check_single_type(paths, geos)
    variant = next((g.name for g in geos if g.name), None)
    log(f"Projection: {kind}" + (f" ({variant})" if variant else ""))

    psize = min(g.psize for g in geos)      # finest scale wins

    tiles = []
    for p, g in zip(paths, geos):
        t = load_tile(p, g, psize, args.erode)
        if t is None:
            log(f"  skipping (no valid pixels): {p}")
            continue
        tiles.append(t)
    if not tiles:
        sys.exit("No usable images")

    if not all(t.gray for t in tiles):
        for t in tiles:
            if t.gray:
                t.img = np.repeat(t.img[..., None], 3, axis=2)
    gray = all(t.gray for t in tiles)

    x_ref, y_ref, W, H = place_tiles(tiles, kind, psize)
    log(f"Canvas {W} x {H} px at {psize:.6f} units/px")

    if not args.no_gain and len(tiles) > 1:
        log("Solving exposure gains...")
        gains = solve_gains(tiles)
        for t, g in zip(tiles, gains):
            t.gain = float(g)
        log(f"  gain range {gains.min():.3f} .. {gains.max():.3f}")

    log("Building seams...")
    label = build_label_map(tiles, H, W)
    log("Blending...")
    levels = max(0, min(args.levels, 8))
    mosaic = multiband_blend(tiles, label, H, W, levels)

    cover = label >= 0
    ys, xs = np.where(cover)
    cy0, cy1, cx0, cx1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    mosaic, cover = mosaic[cy0:cy1, cx0:cx1], cover[cy0:cy1, cx0:cx1]
    h, w = cover.shape

    mosaic = np.clip(mosaic, 0.0, 1.0)
    rgb = np.repeat(mosaic[..., None], 3, axis=2) if gray else mosaic
    out16 = np.zeros((h, w, 4), np.uint16)
    out16[..., :3] = np.round(rgb * 65535.0).astype(np.uint16)
    out16[..., :3][~cover] = 0
    out16[..., 3] = np.where(cover, 65535, 0)
    cv2.imwrite(args.output, out16)
    log(f"Wrote {args.output} ({w} x {h})")

    if args.preview:
        lo, hi = np.percentile(mosaic[cover], (0.5, 99.5))
        prev = np.clip((rgb - lo) / max(hi - lo, 1e-6), 0, 1)
        prev8 = np.round(prev * 255).astype(np.uint8)
        prev8[~cover] = 0
        pp = os.path.splitext(args.output)[0] + "-preview.png"
        cv2.imwrite(pp, prev8)
        log(f"Wrote {pp}")

    # Sidecar metadata for the mosaic, in the same layout as the inputs.
    left = x_ref + cx0 * psize
    top = y_ref + cy0 * psize
    if kind == "cyl":
        left %= 360.0
        if any(g.ox < 0 for g in geos) and left > 180.0:
            left -= 360.0                     # keep the inputs' -180..180 convention
        params = {"Cylindrical": {
            "top_lat": -top,
            "bottom_lat": -(top + h * psize),
            "left_lon": left,
            "right_lon": left + w * psize,    # may pass 360/180 if the mosaic wraps
            "width": int(w), "height": int(h),
            "degrees_per_pixel": psize}}
        pname = "Cylindrical"
    else:
        r = -geos[0].ox
        params = {"Polar": {
            "min_x": left, "max_x": left + w * psize,
            "min_y": top, "max_y": top + h * psize,
            "max_radius": r, "max_edge_length": geos[0].edge,
            **({"eye_height": geos[0].eye_height} if geos[0].eye_height is not None else {}),
            "width": int(w), "height": int(h),
            "units_per_pixel": psize}}
        pname = "Polar"
    meta = {"mosaic": {
        "projection": {"projection": pname, "variant": variant, "parameters": params},
        "sources": [{"file": t.name, "gain": round(t.gain, 5)} for t in tiles]}}
    mp = os.path.splitext(args.output)[0] + "-metadata.json"
    with open(mp, "w") as f:
        json.dump(meta, f, indent=2)
    log(f"Wrote {mp}")


if __name__ == "__main__":
    main()