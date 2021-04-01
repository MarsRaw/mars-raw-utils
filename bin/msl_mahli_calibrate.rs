use mars_raw_utils::{constants, print, vprintln, path, imagebuffer, rgbimage, enums};

#[macro_use]
extern crate clap;

use clap::{Arg, App};


fn main() {
    
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name(constants::param::PARAM_INPUTS)
                        .short(constants::param::PARAM_INPUTS_SHORT)
                        .long(constants::param::PARAM_INPUTS)
                        .value_name("INPUT")
                        .help("Input")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                        .short(constants::param::PARAM_VERBOSE)
                        .help("Show verbose output"))
                    .get_matches();

    let red_scalar = 1.16;
    let green_scalar = 1.0;
    let blue_scalar = 1.05;
    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    let input = matches.value_of(constants::param::PARAM_INPUTS).unwrap();
    
    let mut raw = rgbimage::RgbImage::open(input).unwrap();

    /*
    data = apply_inpaint_fix(data)
    data = apply_lut(data)
    data = apply_flat_field(data)
    data = apply_rad_multiple(data, rad_corr_mult_red, rad_corr_mult_green, rad_corr_mult_blue)

    */
    //if data.shape[0] == 1200 and data.shape[1] == 1632:
    //data = data[16:1200,32:1616]
    if raw.width == 1632 && raw.height == 1200 {
        vprintln!("Cropping...");
        raw.crop(32, 16, 1584, 1184).unwrap();
    }
    
    vprintln!("Inpainting...");
    raw.apply_inpaint_fix(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Decompanding...");
    raw.decompand(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Flatfielding...");
    raw.flatfield(enums::Instrument::MslMAHLI).unwrap();

    vprintln!("Applying color weights...");
    raw.apply_weight(red_scalar, green_scalar, blue_scalar).unwrap();

    vprintln!("Normalizing...");
    raw.normalize_to_16bit_with_max(2033.0).unwrap();

    vprintln!("Writing to disk...");
    raw.save("/home/kgill/Desktop/test.png").unwrap();

}
