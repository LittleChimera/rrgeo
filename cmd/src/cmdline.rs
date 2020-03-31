extern crate time;
extern crate reverse_geocoder;

use reverse_geocoder::{
    Locations,
    ReverseGeocoder,
};

use std::{
    env,
    process::{
        exit,
    },
};

use time::{
    PreciseTime,
};

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        eprint!("Usage: rreverse lat long\n e.g., rreverse 44.962786 -93.344722\n\n");
        exit(1);
    }

    let lat = args[1].parse::<f64>().expect("Couldn't parse latitude");
    let long = args[2].parse::<f64>().expect("Couldn't parse longitude");

    let loc = Locations::from_memory();
    let geocoder = ReverseGeocoder::new(&loc);

    let start = PreciseTime::now();
    let search_result = geocoder.search((lat, long)).expect("Nothing found.");
    let end = PreciseTime::now();
    eprintln!("{} ms to search", start.to(end).num_milliseconds());

    println!("Location: {}", search_result.record);
    println!("Distance: {}", search_result.distance);
}
