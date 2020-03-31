#![deny(missing_docs)]
//! A library for fast, offline reverse geocoding. The location data are from [GeoNames](http://www.geonames.org/).
//!
//! # Usage
//! First, add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! reverse_geocoder = "^1.0.1"
//! ```
//!
//! Next:
//!
//! ```
//! use reverse_geocoder::{Locations, ReverseGeocoder};
//!
//! fn main() {
//!     let loc = Locations::from_memory();
//!     let geocoder = ReverseGeocoder::new(&loc);
//!     let coords = (45.0, 54.0);
//!     let search_result = geocoder.search(coords).expect("Search error.");
//!     println!("Distance {}", search_result.distance);
//!     println!("Record {}", search_result.record);
//! }
//!```

#[macro_use]
extern crate serde_derive;

use kdtree::{distance::squared_euclidean, KdTree};
// use time::Instant;
use std::fmt;
use std::path::PathBuf;
use std::error;

/// A parsed location.
#[derive(Debug, Clone, RustcDecodable, RustcEncodable, Serialize, Deserialize)]
pub struct Record {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
    /// Location name
    pub name: String,
    /// Administrative district 1
    pub admin1: String,
    /// Administrative district 2
    pub admin2: String,
    /// Administrative district 3
    pub admin3: String,
}

/// Search result from querying a lat/long.
pub struct SearchResult<'a> {
    /// Distance away from given lat/long.
    pub distance: f64,
    /// Closest place information.
    pub record: &'a Record,
}

/// A set of location records.
pub struct Locations {
    records: Vec<([f64; 2], Record)>,
}

/// Reverse Geocoder's ErrorKind
#[derive(Debug)]
pub enum ErrorKind {
    /// Couldn't find a result.
    NoResultsFound,
    /// Issue with the underlying k-d tree.
    KdTreeError(kdtree::ErrorKind),
}

impl Locations {
    /// Use the built-in cities.csv.
    pub fn from_memory() -> Locations {
        let mut records = Vec::new();
        let my_str = include_str!("../cities.csv");
        let reader = quick_csv::Csv::from_string(my_str).has_header(true);

        for read_record in reader {
            let record: Record = read_record.unwrap().decode().unwrap();
            records.push(([record.lat, record.lon], record));
        }
        Locations { records: records }
    }

    /// Supply your own path to a CSV file.
    pub fn from_path(path: PathBuf) -> Result<Locations, Box<dyn error::Error>> {
        // let start = Instant::now();
        let mut records = Vec::new();

        let reader = quick_csv::Csv::from_file(path)?.has_header(true);

        for read_record in reader {
            let record: Record = read_record?.decode()?;
            records.push(([record.lat, record.lon], record));
        }

        // let end = Instant::now();

        // println!("{} ms to load cities.csv", (end - start).whole_milliseconds());

        Ok(Locations { records: records })
    }
}

/// A reverse geocoder.
pub struct ReverseGeocoder<'a> {
    tree: KdTree<f64, &'a Record, &'a [f64; 2]>,
}

impl<'a> ReverseGeocoder<'a> {
    /// Create a new reverse geocoder from a set of locations.
    pub fn new(loc: &'a Locations) -> ReverseGeocoder<'a> {
        let mut reverse_geocoder = ReverseGeocoder::<'a> {
            tree: KdTree::with_capacity(2, loc.records.len()),
        };
        reverse_geocoder.initialize(loc);
        reverse_geocoder
    }

    fn initialize(&mut self, loc: &'a Locations) {
        // let start = Instant::now();
        for record in &loc.records {
            self.tree.add(&record.0, &record.1).unwrap();
        }
        // let end = Instant::now();
        // println!("{} ms to build the KdTree", (end - start).whole_milliseconds());
    }

    /// Search for the closest record to a given (latitude, longitude).
    pub fn search(&self, loc: (f64, f64)) -> Result<SearchResult, ErrorKind> {
        let nearest = match self.tree.nearest(&[loc.0, loc.1], 1, &squared_euclidean) {
            Ok(nearest) => nearest,
            Err(error) => return Err(ErrorKind::KdTreeError(error)),
        };
        match nearest.get(0) {
            Some(nearest) => {
                Ok(SearchResult {
                    distance: nearest.0,
                    record: nearest.1,
                })
            },
            None => Err(ErrorKind::NoResultsFound),
        }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}): {}, {}, {}, {}",
            self.lat, self.lon, self.name, self.admin1, self.admin2, self.admin3
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_3_places() {
        let loc = Locations::from_memory();
        let geocoder = ReverseGeocoder::new(&loc);
        let slp = geocoder.search((44.962786, -93.344722)).unwrap();

        assert_eq!(slp.record.name, "Saint Louis Park");

        // [44.894519, -93.308702] is 60 St W @ Penn Ave S, Minneapolis, Minnesota; however, this is physically closer to Richfield
        let mpls = geocoder.search((44.894519, -93.308702)).unwrap();
        assert_eq!(mpls.record.name, "Richfield");

        // [44.887055, -93.334204] is HWY 62 and Valley View Road, whish is in Edina
        let edina = geocoder.search((44.887055, -93.334204)).unwrap();
        assert_eq!(edina.record.name, "Edina");
    }

    #[test]
    fn it_loads_locations_from_a_path() -> Result<(), Box<dyn error::Error>> {
        let loc = Locations::from_path("./cities.csv".into())?;
        ReverseGeocoder::new(&loc);

        Ok(())
    }

    #[test]
    fn it_returns_an_error_given_an_infinite_coordinate() {
        let loc = Locations::from_memory();
        let geocoder = ReverseGeocoder::new(&loc);
        let fail = geocoder.search((std::f64::INFINITY, 54.0));
        assert!(fail.is_err());
    }
}
