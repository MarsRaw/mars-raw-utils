
use mars_raw_utils::{constants, print, vprintln, jsonfetch};


fn main() {

    print::set_verbose(true);

    println!("Hello, world!");

    let params = vec![
        vec!["feed", "raw_images"],
        vec!["category", "mars2020"],
        vec!["feedtype", "json"],
        vec!["num", "2"],
        vec!["page", "0"],
        vec!["order", "sol desc"],
        vec!["search", "NAVCAM_RIGHT"],
        vec!["condition_2", "43:sol:gte"],
        vec!["condition_3", "46:sol:lte"]
    ];

    let uri = constants::url::M20_RAW_WEBSERVICE_URL;

    let mut req = jsonfetch::JsonFetcher::new(uri);

    for p in params {
        req.param(p[0], p[1]);
    }

    let json_res = req.fetch().unwrap();
    vprintln!("Number of images found: {}", json_res["images"].len());
    //let num_images = json_res["images"].len();


}
