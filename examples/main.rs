extern crate street_index;

use street_index::*;

fn main() {
    let mut grid = Grid::new(
            Bbox { 
                width: Millimeter(200.0), 
                height: Millimeter(200.0) 
            },
            GridConfig {
                cell_width: Millimeter(20.0),
                cell_height: Millimeter(20.0),
            });

    grid.insert_font(StreetNameRect {
        street_name: String::from("Canterbury Road"),
        x_from_left: Millimeter(30.0),
        width: Millimeter(50.0),
        y_from_top: Millimeter(30.0),
        height: Millimeter(8.0),
    });

    let deduplicated = DeduplicatedRoads::from_streets(&grid.into_street_names());
    let (processed, unprocessed) = deduplicated.process();

    println!("processed:\r\n{}", processed.to_csv());
    println!("unprocessed:\r\n{}", unprocessed.to_csv());
}