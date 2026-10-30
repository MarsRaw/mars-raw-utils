use crate::subs::runnable::RunnableSubcommand;
use anyhow::{anyhow, Result};
use clap::Parser;

use mars_raw_utils::dsnstatus::{
    self, Dish as DsnDish, DishHashMap, DownSignal,
    DsnItem::{Dish, Other, Station},
    SpacecraftHashMap, Station as DsnStation, Target, UpSignal,
};
use num_format::{Locale, ToFormattedString};

fn print_station(station: &DsnStation) {
    println!(
        "Station: {} -- {}",
        station.friendly_name,
        station.time_utc.to_rfc2822()
    );
}

fn print_upsignal(up: &UpSignal, spacecraft_map: &SpacecraftHashMap) {
    if up.active {
        match spacecraft_map.get(&up.spacecraft.to_lowercase()) {
            Some(spacecraft) => println!(
                "          Up: {} ({})",
                spacecraft.friendly_name, up.spacecraft
            ),
            None => println!("          Up: {}", up.spacecraft),
        };

        println!(
            "              Active: {}",
            match up.active {
                true => "Yes",
                false => "No",
            }
        );

        println!("              Signal Type: {}", up.signal_type);
        if up.data_rate > 0.0 {
            println!("              Data Rate: {} kb/sec", up.data_rate / 1000.0);
        }

        if up.frequency > 0 {
            println!("              Frequency: {}", up.frequency);
        }

        println!("              Power Transmitted: {} dBm", up.power);

        println!("              Band: {}", up.band);
    } else {
        match spacecraft_map.get(&up.spacecraft.to_lowercase()) {
            Some(spacecraft) => println!(
                "          Up: {} ({}) - Inactive",
                spacecraft.friendly_name, up.spacecraft
            ),
            None => println!("          Up: {} - Inactive", up.spacecraft),
        };
    }
}

fn print_downsignal(down: &DownSignal, spacecraft_map: &SpacecraftHashMap) {
    if down.active {
        match spacecraft_map.get(&down.spacecraft.to_lowercase()) {
            Some(spacecraft) => println!(
                "          Down: {} ({})",
                spacecraft.friendly_name, down.spacecraft
            ),
            None => println!("          Down: {}", down.spacecraft),
        };
        println!(
            "              Active: {}",
            match down.active {
                true => "Yes",
                false => "No",
            }
        );

        println!("              Signal Type: {}", down.signal_type);

        if down.data_rate > 0.0 {
            println!(
                "              Data Rate: {} kb/sec",
                down.data_rate / 1000.0
            );
        }

        if down.frequency > 0 {
            println!("              Frequency: {}", down.frequency);
        }

        println!("              Power Received: {} dBm", down.power);

        println!("              Band: {}", down.band);
    } else {
        match spacecraft_map.get(&down.spacecraft.to_lowercase()) {
            Some(spacecraft) => println!(
                "          Down: {} ({}) - Inactive",
                spacecraft.friendly_name, down.spacecraft
            ),
            None => println!("          Down: {} - Inactive", down.spacecraft),
        };
    }
}

fn print_target(target: &Target, spacecraft_map: &SpacecraftHashMap) {
    match spacecraft_map.get(&target.name.to_lowercase()) {
        Some(spacecraft) => println!(
            "          Target: {} ({})",
            spacecraft.friendly_name, target.name
        ),
        None => println!("          Target: {}", target.name),
    };

    if target.upleg_range >= 0 {
        println!(
            "              Up Range: {} km",
            target.upleg_range.to_formatted_string(&Locale::en)
        );
    }

    if target.downleg_range >= 0 {
        println!(
            "              Down Range: {} km",
            target.downleg_range.to_formatted_string(&Locale::en)
        );
    }

    if target.rtlt > 0 {
        println!("              Round-trip Light Time: {}", target.rtlt);
    }
}

fn print_dish(dish: &DsnDish, spacecraft_map: &SpacecraftHashMap, dish_map: &DishHashMap) {
    match dish_map.get(&dish.name.to_lowercase()) {
        Some(dish_cfg) => println!(
            "  Dish: {} ({})",
            dish_cfg.friendly_name, dish_cfg.dish_type
        ),
        None => println!("  Dish: {}", dish.name),
    };

    //println!("  Dish: {}", dish.name);
    println!("      Activity: {}", dish.activity);
    println!("      Azimuth: {} deg", dish.azimuth_angle);
    println!("      Elevation: {} deg", dish.elevation_angle);
    println!("      Wind Speed: {} km/hr", dish.wind_speed);

    for up in &dish.up_signal {
        print_upsignal(up, spacecraft_map);
    }

    for down in &dish.down_signal {
        print_downsignal(down, spacecraft_map);
    }

    for target in &dish.target {
        print_target(target, spacecraft_map);
    }
}

#[derive(Parser)]
#[command(author, version, about = "Retrieve DSN status information", long_about = None)]
pub struct DsnStatus {}

impl RunnableSubcommand for DsnStatus {
    async fn run(&self) -> Result<()> {
        let dsn = match dsnstatus::fetch_dsn_status().await {
            Ok(dsn) => dsn,
            Err(why) => return Err(anyhow!("Failed to fetch DSN status: {:?}", why)),
        };

        let dsn_config = match dsnstatus::fetch_dsn_config().await {
            Ok(dsn_config) => dsn_config,
            Err(why) => return Err(anyhow!("Failed to retrieve DSN config: {:?}", why)),
        };

        let spacecraft_map = dsn_config.create_spacecraft_hashmap();
        let dish_map = dsn_config.create_dish_hashmap();

        for item in &dsn.items {
            match item {
                Station(station) => print_station(station),
                Dish(dish) => print_dish(dish, &spacecraft_map, &dish_map),
                Other => {}
            };
        }

        Ok(())
    }
}
