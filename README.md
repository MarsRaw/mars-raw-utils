# Mars Raw Image Utilities
A set of utilities for processing and calibration of imagery from either the Curiosity or Perseverance rovers. Meant to be used on publicly available images. 

Implemented calibration steps include (varying per instrument):

| Mission    |     Camera  | Decompand | Debayer | Inpaint      | Flats  | HPC*   |
| ---------- |:-----------:|:---------:|:-------:|:------------:|:------:|:------:|
| MSL        | MastCam     | &#9745;   | &#9745; |              |        |        |
| MSL        | MAHLI       | &#9745;   |         | &#9745;      | &#9745;|        |
| MSL        | NavCam      |           |         | &#9745;      | &#9745;| &#9745;|
| MSL        | Rear Haz    |           |         | &#9745;      | &#9745;| &#9745;|
| MSL        | Front Haz   |           |         | &#9745;      | &#9745;| &#9745;|
| Mars2020   | Mastcam-Z   | &#9745;   | &#9745; | &#9745;      |        |        |
| Mars2020   | NavCam      |           | &#9745; |              |        |        |
| Mars2020   | Rear Haz    |           | &#9745; |              |        |        |
| Mars2020   | Front Haz   |           | &#9745; |              |        |        |
| Mars2020   | Watson      |           |         | &#9745;      |        |        |
| InSight    | IDC         | &#9745;   |         |              |        |        |
| InSight    | ICC         | &#9745;   |         |              |        |        |


\* Hot pixel detection and correction


Additional instruments will be implemented more or less whenever I get to them...

## Building from source:
So far I've only tested building on Ubuntu 20.04, both natively and within the Windows Subsystem for Linux on Windows 10. Within the project folder, the software can be built for testing via `cargo build` and individual binaries can be run in debug mode via, for example, `cargo run --bin m20_fetch_raw -- -i`

To build successfully on Ubuntu, you'll likely need the following packages installed via apt:
* build-essential
* libopencv-dev 
* libssl-dev 
* libclang1 
* libclang-dev 
* librust-clang-sys-dev 
* librust-clang-sys+runtime-dev 
* clang-tools 

The dockerfile demonstrates a method for building an installable debian package, or you can use the container itself:

```
docker build -t mars_raw_utils .
docker run --name mars_raw_utils -dit mars_raw_utils
docker exec -it mars_raw_utils bash
```

Builds for RPM, MacOSX and Windows are in the plan.


## Mars Science Laboratory (Curiosity):
### Fetch Raws:
```
SAGE:
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
### Engineering Cameras (Navcam, FHAZ, RHAZ):
```
SAGE:
    msl_ecam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Raw color, skip ILT (not currently used)
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>          Blue weight
    -G, --green <GREEN>        Green weight
    -i, --inputs <INPUT>...    Input
    -R, --red <RED>            Red weight
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
