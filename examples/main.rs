extern crate street_index;

use street_index::*;
use gridconfig::number_to_alphabet_value;

fn main() {
    /*
    let road_names = [
        InputStreetValue {
            street_name: StreetName(String::from("Valley View Road")),
            position: GridPosition {
                column: String::from("A"),
                row: 4,
            }
        },
        InputStreetValue {
            street_name: StreetName(String::from("Valley View Road")),
            position: GridPosition {
                column: String::from("A"),
                row: 5,
            }
        },
        InputStreetValue {
            street_name: StreetName(String::from("Valley View Road")),
            position: GridPosition {
                column: String::from("B"),
                row: 6,
            }
        },
    ];

    let deduplicated = DeduplicatedRoads::from_streets(&road_names);
    let (processed, unprocessed) = deduplicated.process();
    println!("processed:\r\n{}\r\n", processed.to_csv());
    println!("unprocessed:\r\n{}", unprocessed.to_csv());
    */

    println!("{}", number_to_alphabet_value(3356));
}