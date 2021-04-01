# Mars Raw Image Utilities
A set of utilities for processing and calibration of imagery from either the Curiosity or Perseverance rovers. Meant to be used on publicly available images. 

Implemented calibration steps include (varying per instrument):

| Step                         | MAHLI     | Mastcam   | Mastcam-Z |
| ---------------------------- |:---------:|:---------:|:---------:|
| Decompanding                 | &#9745;   | &#9745;   | &#9745;   |
| Debayer (demosaicking)       |           | &#9745;   |           |
| Blemish repair (inpainting)  | &#9745;   |           | &#9745;   |
| Flatfielding                 | &#9745;   |           |           |
| Color weight correction      | &#9745;   | &#9745;   | &#9745;   |


Additional instruments will be implemented more or less whenever I get to them...


## Mars Science Laboratory (Curiosity):
### Fetch Raws:
...

### MAHLI Calibration:
```
USAGE:
    msl_mahli_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
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
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>                                      Blue weight
    -c, --color_noise_reduction <COLOR_NOISE_REDUCTION>    Color noise reduction amount in pixels
    -G, --green <GREEN>                                    Green weight
    -i, --inputs <INPUT>...                                Input
    -R, --red <RED>                                        Red weight
```
...
## Mars 2020 (Perseverance):
### Fetch Raws:
...

### MastCam-Z:
```
USAGE:
    m20_zcam_calibrate [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>                                      Blue weight
    -c, --color_noise_reduction <COLOR_NOISE_REDUCTION>    Color noise reduction amount in pixels
    -G, --green <GREEN>                                    Green weight
    -i, --inputs <INPUT>...                                Input
    -R, --red <RED>                                        Red weight
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

