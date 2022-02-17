# Mars Raw Image Utilities
[![Rust](https://github.com/kmgill/mars-raw-utils/actions/workflows/rust.yml/badge.svg)](https://github.com/kmgill/mars-raw-utils/actions/workflows/rust.yml)

A set of utilities for processing and calibration of imagery from either the Curiosity or Perseverance rovers. Meant to be used on publicly available images. 

Implemented calibration steps include (varying per instrument):

| Mission    |     Camera  | Decompand | Debayer | Inpaint      | Flats  | HPC*   |
| ---------- |:-----------:|:---------:|:-------:|:------------:|:------:|:------:|
| MSL        | MastCam     | &#9745;   | &#9745; |              |        |        |
| MSL        | MAHLI       | &#9745;   |         | &#9745;      | &#9745;|        |
| MSL        | NavCam      |           |         | &#9745;      | &#9745;| &#9745;|
| MSL        | Rear Haz    |           |         | &#9745;      | &#9745;| &#9745;|
| MSL        | Front Haz   |           |         | &#9745;      | &#9745;| &#9745;|
| MSL        | ChemCam RMI |           |         |              | &#9745;|        |
| Mars2020   | Mastcam-Z   | &#9745;   | &#9745; | &#9745;      |        |        |
| Mars2020   | NavCam      |           | &#9745; |              |        |        |
| Mars2020   | Rear Haz    |           | &#9745; |              |        |        |
| Mars2020   | Front Haz   |           | &#9745; |              |        |        |
| Mars2020   | Watson      | &#9745;   | &#9745; | &#9745;      |        |        |
| Mars2020   | SuperCam    |           | &#9745; |              | &#9745;|        |
| Ingenuity  | Nav         |           |         |              | &#9745;|        |
| Ingenuity  | Color       |           |         |              | &#9745;|        |
| InSight    | IDC         | &#9745;   |         |              | &#9745;|        |
| InSight    | ICC         | &#9745;   |         |              | &#9745;|        |


\* Hot pixel detection and correction


Additional instruments will be implemented more or less whenever I get to them...

## Building from source:
A working Rust (https://www.rust-lang.org/) installation is required for building.

So far I've only tested building on Ubuntu 20.04, both natively and within the Windows Subsystem for Linux on Windows 10 and on MacOSX Catalina. Within the project folder, the software can be built for testing via `cargo build` and individual binaries can be run in debug mode via, for example, `cargo run --bin m20_fetch_raw -- -i`

To build successfully on Linux, you'll likely need the following packages installed via apt:
* libssl-dev (Ubuntu)
* openssl-devel (RHEL, CentOS, Fedora)

### Clone from git:
```
git clone git@github.com:kmgill/mars-raw-utils.git
cd mars-raw-utils/
git submodule init
git submodule update
```

### Install via cargo:
```
cargo install --path .
export MARS_RAW_DATA=$PWD/mars-raw-utils-data/caldata
```
NOTE: You'll want to set $MARS_RAW_DATA in ~/.bash_profile using the absolute path.

### Install via apt (Debian, Ubuntu, ...):
```
cargo install cargo-deb
cargo deb
sudo apt install ./target/debian/mars_raw_utils_0.1.3_amd64.deb
```
NOTE: Adjust the output debian package filename to what is outputted by build.

### Install via rpm (RHEL, CentOS, Fedora, ...)
```
cargo install cargo-rpm
cp -v mars-raw-utils-data/caldata/* .rpm/
cargo rpm build -v
rpm -ivh target/release/rpmbuild/RPMS/x86_64/mars_raw_utils-0.1.3-1.el8.x86_64.rpm
```
NOTE: Adjust the output rpm package filename to what is outputted by build.

### Docker:
The dockerfile demonstrates a method for building an installable debian package, or you can use the container itself:

```
docker build -t mars_raw_utils .
docker run --name mars_raw_utils -dit mars_raw_utils
docker exec -it mars_raw_utils bash
```

Builds for MacOSX (maybe via Homebrew?) and Windows are in the plan. Though the project has built and run from MacOSX and Windows, I haven't worked out the installation method in a way that handles the calibration data.

## Specifying Calibration Data Location:
By default, if the software is installed using the .deb file in Debian/Ubuntu, the calibration files will be located in `/usr/share/mars_raw_utils/data/`. In Homebrew on MacOS, they will be located in `/usr/local/share/mars_raw_utils/data/`. For installations using `cargo install --path .` or custom installations, you can set the calibration file directory by using the `MARS_RAW_DATA` environment variable. The variable will override the default locations (if installed via apt or rpm), as well.

## Mars Science Laboratory (Curiosity):
### Fetch Raws:
```
USAGE:
    msl_fetch_raw [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -l, --list          Don't download, only list results
    -t, --thumbnails    Download thumbnails in the results
    -v                  Show verbose output
    -V, --version       Prints version information

OPTIONS:
    -c, --camera <camera>...    M20 Camera Instrument(s)
    -M, --maxsol <maxsol>       Ending Mission Sol
    -m, --minsol <minsol>       Starting Mission Sol
    -n, --num <num>             Max number of results
    -p, --page <page>           Results page (starts at 1)
    -S, --seqid <seqid>         Specific sequence id or substring
    -s, --sol <sol>             Mission Sol
```

#### Examples:

Show available instruments:
```
msl_fetch_raw -i
```

List what's available for Mastcam on sol 3113: (remove the `-l` to download the images)
```
msl_fetch_raw -c MASTCAM -s 3113 -l
```

List what's available for NAV_RIGHT between sols 3110 and 3112: (remove the `-l` to download the images)
```
msl_fetch_raw -c NAV_RIGHT -m 3110 -M 3112 -l
```

Download NAV_RIGHT during sols 3110 through 3112, filtering for sequence id NCAM00595:
```
msl_fetch_raw -c NAV_RIGHT -m 3110 -M 3112 -S NCAM00595
```

### MAHLI Calibration:
```
USAGE:
    msl_mahli_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```
#### Recommended Color Correction Multiples:
* RED: 1.16
* GREEN: 1.00
* BLUE: 1.05

#### Examples:
Calibrate a directory of JPEGs, applying color correction values:
```
msl_mahli_calibrate -i *jpg -v -R 1.16 -G 1.00 -B 1.05
```

### MastCam:
```
USAGE:
    msl_mcam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>                                      Blue weight
    -c, --color_noise_reduction <COLOR_NOISE_REDUCTION>    Color noise reduction amount in pixels
    -G, --green <GREEN>                                    Green weight
    -i, --inputs <INPUT>...                                Input
    -R, --red <RED>                                        Red weight
```

#### Recommended Color Correction Multiples:
* RED: 1.20
* GREEN: 1.00
* BLUE: 1.26

#### Examples:
Calibrate a directory of JPEGs, applying color correction values:
```
msl_mcam_calibrate -i *jpg -v -R 1.20 -G 1.0 -B 1.26
```

Calibrate a directory of JPEGs, skipping ILT conversion (decompanding):
```
msl_mcam_calibrate -i *jpg -v -r
```

Calibrate a directory of JPEGs, applying color noise reduction with a chroma blur radius of 21 pixels:
```
msl_mcam_calibrate -i *jpg -v -c 21
```

### Engineering Cameras (Navcam, FHAZ, RHAZ):
```
USAGE:
    msl_ecam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -n               Only new images. Skipped processed images.
    -r, --raw        Raw color, skip ILT (not currently used)
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>                  Blue weight
    -G, --green <GREEN>                Green weight
    -t, --hpc_threshold <THRESHOLD>    Hot pixel correction variance threshold
    -i, --inputs <INPUT>...            Input
    -R, --red <RED>                    Red weight
```

#### Examples:
Calibrate a directory of JPEGs:
```
msl_ecam_calibrate -i *jpg -v
```

Calibrate a directory of JPEGs, apply a hot pixel detection with a threshold of 2.5 standard deviations:
```
msl_ecam_calibrate -i *jpg -v -t 2.5
```

### ChemCam RMI:
```
USAGE:
    msl_ccam_calibrate [FLAGS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -n               Only new images. Skipped processed images.
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -i, --inputs <INPUT>...    Input
```

## Mars 2020 (Perseverance):
### Fetch Raws:
```
USAGE:
    m20_fetch_raw [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -l, --list          Don't download, only list results
    -t, --thumbnails    Download thumbnails in the results
    -v                  Show verbose output
    -V, --version       Prints version information

OPTIONS:
    -c, --camera <camera>...    M20 Camera Instrument(s)
    -M, --maxsol <maxsol>       Ending Mission Sol
    -m, --minsol <minsol>       Starting Mission Sol
    -n, --num <num>             Max number of results
    -p, --page <page>           Results page (starts at 1)
    -S, --seqid <seqid>         Specific sequence id or substring
    -s, --sol <sol>             Mission Sol
```

### MastCam-Z:
```
USAGE:
    m20_zcam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```
### Watson:
```
USAGE:
    m20_watson_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```
### Engineering Cameras (Navcam, FHAZ, RHAZ):
```
USAGE:
    m20_ecam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```
### SuperCam
```
USAGE:
    m20_scam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -n               Only new images. Skipped processed images.
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```

### Ingenuity Nav Camera:
```
USAGE:
    m20_hnav_calibrate [FLAGS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -n               Only new images. Skipped processed images.
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -i, --inputs <INPUT>...    Input
```

### Ingenuity Color Camera (RTE):
```
USAGE:
    m20_hrte_calibrate [FLAGS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -n               Only new images. Skipped processed images.
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -i, --inputs <INPUT>...    Input
```

## InSight
### Fetch Raws:
...
### Instrument Context Camera (ICC):
```
USAGE:
    nsyt_icc_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```

### Instrument Deployment Camera (IDC):
```
USAGE:
    nsyt_idc_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
```

## Hot Pixel Correction Filter
Attempt at hot pixel detection and removal. 

Method:

For each pixel (excluding image border pixels):
 1. Compute the standard deviation of a window of pixels (3x3, say)
 1. Compute the z-score for the target pixel
 1. If the z-score exceeds a threshold variance (example: 2.5) from the mean we replace the pixel value with a median filter

```
USAGE:
    hpc_filter [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -t, --hpc_threshold <THRESHOLD>    Hot pixel correction variance threshold
    -w, --hpc_window <WINDOW_SIZE>     Hot pixel correction window size
    -i, --inputs <INPUT>...            Input
```

## Inpainting Filter
Applies a basic inpainting filter on a set of input images. Inpainting regions need to be marked in red (rgb 255, 0, 0).
```
USAGE:
    inpaint_filter [FLAGS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -i, --inputs <INPUT>...    Input
```

## Upscale Experiment
An experiment in smooth image upscaling using the median-based inpainting algorithm.

```
USAGE:
    upscale [FLAGS] --factor <FACTOR> --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -f, --factor <FACTOR>      Scale factor
    -i, --inputs <INPUT>...    Input
```

## Crop
```
USAGE:
    crop [FLAGS] --crop <WINDOW_SIZE> --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -c, --crop <WINDOW_SIZE>    Crop as x,y,width,height
    -i, --inputs <INPUT>...     Input
```

## Debayer
Apply Malvar Demosaicking (Debayer) on a grayscale bayer-pattern image. Optionally apply a color noise reduction.
```
USAGE:
    debayer [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -c, --color_noise_reduction <COLOR_NOISE_REDUCTION>    Color noise reduction amount in pixels
    -i, --inputs <INPUT>...                                Input
```


## Levels
Apply levels adjustments to an image. Analogous to 'Levels' in Photoshop or GIMP. 
```
USAGE:
    levels [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -b, --blacklevel <BLACK_LEVEL>    Black level
    -g, --gamma <PARAM_GAMMA>         Gamma
    -i, --inputs <INPUT>...           Input
    -w, --whitelevel <WHITE_LEVEL>    White level
```

## Change Detection (Dust devils, clouds)
Calculates a per-frame differential from a mean across a series of images. Intended for use with MSL and Mars2020 dust devil movies and sky surveys. Optional options are for contrast enhancement through Photoshop-like black level, white level, and gamma. 

```
USAGE:
    diffgif [FLAGS] [OPTIONS] --inputs <INPUT>... --output <OUTPUT>

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -b, --blacklevel <BLACK_LEVEL>    Black level
    -B, --blur <PARAM_BLUR>           Gaussian blur kernel size on differential output
    -d, --delay <PARAM_DELAY>         Interframe delay in increments of 10ms
    -g, --gamma <PARAM_GAMMA>         Gamma
    -i, --inputs <INPUT>...           Input
    -o, --output <OUTPUT>             Output
    -w, --whitelevel <WHITE_LEVEL>    White level
```

### Examples
#### Dust Devils, MSL Sol 3372, Seq id NCAM00595:
```
msl_fetch_raw -c NAV_RIGHT_B -s 3372 -S NCAM00595

msl_ecam_calibrate -i *JPG -v -t 2.0

diffgif -i *NCAM00595*-rjcal.png -o DustDevilMovie_Sol3372.gif -v -b 0 -w 2.0 -g 2.5 -B 1.5 -d 20
```
#### Cloud motion and shadows, MSL Sol 3325, Seq id NCAM00556:
```
msl_fetch_raw -c NAV_RIGHT -s 3325

msl_ecam_calibrate -i *JPG -v -t 2.0

diffgif -i *NCAM00556*-rjcal.png -o CloudShadow_3325.gif -v -b 0 -w 1.0 -g 2.5 -B 1.5 -d 20
```
#### Clouds, zenith movie, MSL Sol 3325, Seq id NCAM00551:
```
msl_fetch_raw -c NAV_RIGHT -s 3325

msl_ecam_calibrate -i *JPG -v -t 2.0

diffgif -i *NCAM00551*-rjcal.png -o CloudZenith_3325.gif -v -b 0 -w 3.0 -g 1.0 -B 1.5 -d 20
```

## Mission Dates
Mission time and sol are available for MSL, Mars2020, and InSight via `msl_date`, `m20_date`, and `nsyt_date`, respectively. 

Currently, the output provides valules for the Mars Sol Date, coordinated Mars time, mission sol, mission time (LMST), local true color time, and areocentric solar longitude. The algorithm used for the calculation is based on James Tauber's marsclock.com and is exposed via `time::get_lmst()`.

Example Output:
```
$ msl_date
Mars Sol Date:          52391.26879394437
Coordinated Mars Time:  06:27:03.797
Mission Sol:            3122
Mission Time:           15:36:49.805 LMST
Local True Solar Time:  15:29:37.673 LTST
Solar Longitude:        47.04093399663567

$ m20_date
Mars Sol Date:          52391.270293050664
Coordinated Mars Time:  06:29:13.320
Mission Sol:            87
Mission Time:           11:38:56.520 LMST
Local True Solar Time:  11:31:44.417 LTST
Solar Longitude:        47.04161842268443

$ nsyt_date 
Mars Sol Date:          52391.27048977531
Coordinated Mars Time:  06:29:30.317
Mission Sol:            880
Mission Time:           15:31:59.933 LMST
Local True Solar Time:  15:24:47.833 LTST
Solar Longitude:        47.041708238462114
```

## References:

Bell, J. F. et al. (2017), The Mars Science Laboratory Curiosity rover
Mastcam instruments: Preflight and in‐flight calibration, validation,
and data archiving, Earth and Space Science, 4, 396– 452,
doi:10.1002/2016EA000219.
https://doi.org/10.1002/2016EA000219


Hayes, A.G., Corlies, P., Tate, C. et al.
Pre-Flight Calibration of the Mars 2020 Rover Mastcam Zoom (Mastcam-Z)
Multispectral, Stereoscopic Imager. Space Sci Rev 217, 29 (2021).
https://doi.org/10.1007/s11214-021-00795-x


Edgett, K.S., Yingst, R.A., Ravine, M.A. et al.
Curiosity’s Mars Hand Lens Imager (MAHLI) Investigation.
Space Sci Rev 170, 259–317 (2012).
https://doi.org/10.1007/s11214-012-9910-4


Edgett, K. S., M. A. Caplinger, J. N. Maki, M. A. Ravine, F. T. Ghaemi, S. McNair, K. E. Herkenhoff,
B. M. Duston, R. G. Willson, R. A. Yingst, M. R. Kennedy, M. E. Minitti, A. J. Sengstacken, K. D. Supulver,
L. J. Lipkaman, G. M. Krezoski, M. J. McBride, T. L. Jones, B. E. Nixon, J. K. Van Beek, D. J. Krysak, and R. L. Kirk
(2015) Curiosity’s robotic arm-mounted Mars Hand Lens Imager (MAHLI): Characterization and calibration status,
MSL MAHLI Technical Report 0001 (version 1: 19 June 2015; version 2: 05 October 2015).
doi:10.13140/RG.2.1.3798.5447
https://doi.org/10.13140/RG.2.1.3798.5447


Bell, J. F. et al. (2017), The Mars Science Laboratory Curiosity rover
Mastcam instruments: Preflight and in‐flight calibration, validation,
and data archiving, Earth and Space Science, 4, 396– 452,
doi:10.1002/2016EA000219.
https://doi.org/10.1002/2016EA000219


Deen, R., Zamani, P., Abarca, H., Maki, J. InSight (NSYT)
Software Interface Specification Camera Experiment Data Record (EDR) and Reduced Data Record (RDR) Data
Products (version 3.3: 26 June 2019)
https://pds-imaging.jpl.nasa.gov/data/nsyt/insight_cameras/document/insight_cameras_sis.pdf


Edgett, Kenneth & Caplinger, Michael & Ravine, Michael. (2019). Mars 2020 Perseverance SHERLOC WATSON Camera Pre-delivery Characterization and Calibration Report. 10.13140/RG.2.2.18447.00165. 
https://www.researchgate.net/publication/345959204_Mars_2020_Perseverance_SHERLOC_WATSON_Camera_Pre-delivery_Characterization_and_Calibration_Report


Maurice, Sylvestre & Wiens, R. & Mouélic, S. & Anderson, R. & Beyssac, O. & Bonal, L. & Clegg, S. & Deflores, L. & Dromart, G. & Fischer, W. & Forni, O. & Gasnault, O. & Grotzinger, J. & Johnson, Jordanlee & Martínez-Frías, Jesús & Mangold, N. & McLennan, S. & Montmessin, F. & Rull, Fernando & Sharma, Shiv. (2015). The SuperCam Instrument for the Mars2020 Rover. European Planetary Science Congress 2015. 10. 
https://www.researchgate.net/publication/283271532_The_SuperCam_Instrument_for_the_Mars2020_Rover


J. -M. Reess, Marion Bonafous, L. Lapauw, O. Humeau, T. Fouchet, P. Bernardi, Ph. Cais, M. Deleuze, O. Forni, S. Maurice, S. Robinson, R. C. Wiens, "The SuperCam infrared instrument on the NASA MARS2020 mission: performance and qualification results," Proc. SPIE 11180, International Conference on Space Optics — ICSO 2018, 1118037 (12 July 2019); 
https://doi.org/10.1117/12.2536034


Wiens, R.C., Maurice, S., Barraclough, B. et al. The ChemCam Instrument Suite on the Mars Science Laboratory (MSL) Rover: Body Unit and Combined System Tests. Space Sci Rev 170, 167–227 (2012). 
https://doi.org/10.1007/s11214-012-9902-4


O. Gasnault, S. Maurice, R. C. Wiens, S. Le Mouélic, W. W. Fischer, P. Caïs, K. McCabe, J.-M. Reess, and C. Virmontois 
"SUPERCAM REMOTE MICRO-IMAGER ON MARS 2020." - 46th Lunar and Planetary Science Conference (2015).
https://www.hou.usra.edu/meetings/lpsc2015/pdf/2990.pdf


Telea, Alexandru. (2004). An Image Inpainting Technique Based on the Fast Marching Method. Journal of Graphics Tools. 9. 10.1080/10867651.2004.10487596. 
https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method


Malvar, Henrique & He, Li-wei & Cutler, Ross. (2004). High-quality linear interpolation for demosaicing of Bayer-patterned color images. Acoustics, Speech, and Signal Processing, 1988. ICASSP-88., 1988 International Conference on. 3. iii - 485. 10.1109/ICASSP.2004.1326587. 
https://www.researchgate.net/publication/4087683_High-quality_linear_interpolation_for_demosaicing_of_Bayer-patterned_color_images


Getreuer, Pascal. (2011). Malvar-He-Cutler Linear Image Demosaicking. Image Processing On Line. 1. 10.5201/ipol.2011.g_mhcd. 
https://www.researchgate.net/publication/270045976_Malvar-He-Cutler_Linear_Image_Demosaicking


Maki, J.N., Gruel, D., McKinney, C. et al. The Mars 2020 Engineering Cameras and Microphone on the Perseverance Rover: A Next-Generation Imaging System for Mars Exploration. Space Sci Rev 216, 137 (2020). 
https://doi.org/10.1007/s11214-020-00765-9


Malin, M. C., et al. (2017), The Mars Science Laboratory (MSL) Mast cameras and Descent imager: Investigation and instrument descriptions, Earth and Space Science, 4, 506– 539, doi:10.1002/2016EA000252.
https://doi.org/10.1002/2016EA000252


Di, K., and Li, R. (2004), CAHVOR camera model and its photogrammetric conversion for planetary applications, J. Geophys. Res., 109, E04004, doi:10.1029/2003JE002199.
https://doi.org/10.1029/2003JE002199