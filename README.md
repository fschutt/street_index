# street_index

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status Linux / macOS](https://travis-ci.org/fschutt/street_index.svg?branch=master)](https://travis-ci.org/fschutt/street_index)
[![Build status Windows](https://ci.appveyor.com/api/projects/status/p487hewqh6bxeucv?svg=true)](https://ci.appveyor.com/project/fschutt/street_index)
[![Rust Compiler Version](https://img.shields.io/badge/rustc-1.26%20stable-blue.svg)]()

This library contains utility functions for generating a street index.
How it works is fairly simple: You give it a grid (right now limited 
to a rectangular grid) on a page, and add `StreetNameRect`s. Each 
`StreetNameRect` contains the String for the street / road name as well
as the extents of the laid out String on the map.

The `Grid` takes care of assigning a grid position to your street name 
such as `"Canterbury Road => A2"`. Since usually maps have the problem
of having duplicated road names (which is not what you'd want in a street 
index), you can create a `DeduplicatedRoadNames::from_streets`, which will
deduplicate all road names.

The problem generally arises when street names are ambigouus. For example,
a street that appears in two locations on the map (such as a city having 
the same street name as a neighbouring city). Because of this, street name
processing can't be fully automated, since there are always weird edge cases 
to worry about. However, 90% of roads aren't like that.

Because of this limitation `DeduplicatedRoadNames::process()` gives you
two types of roads back: `ProcessedRoadName` is for roads that span only
1 or 2 grid cells (i.e. `"Canterbury Road" => A9`, `"Canterbury Road" => A9-A10`).
In these cases (which cover 90% of street index names), the mapping is not
ambigouus.

`UnprocessedRoadName` is for anything else (e.g. `"Canterbury Road" => [A9, A10, E1, E2]`. 
Usually these roads need to be manually reviewed - it could likely be that 
there are two roads `"Canterbury Road" => A9-10;E1-E2`, but it could also
be that the road is just one road and part of it is just clipped off the map,
in which case you'd write `"Canterbury Road" => A9-E2`.  

For cartographic purposes, usually you want the output in CSV format, so
that your graphic designer can paste the street index into InDesign / 
Illustrator for the final map layout. Both `UnprocessedRoads` and 
`ProcessedRoads` have a simple `.to_csv` function for easy export.

## Example

```rust
extern crate street_index;

use street_index::prelude::*;

fn main() {
	// Create a grid, with the page extensions being 200 x 200 millimeter
	// Each cell is 20x20 millimeter large (usually 50x50 is recommended, though)
    let mut grid = Grid::new(
            Bbox { 
                width: Millimeter(200.0), 
                height: Millimeter(200.0) 
            },
            GridConfig {
                cell_width: Millimeter(20.0),
                cell_height: Millimeter(20.0),
            });

    // You will have to calculate the street name boundaries yourself, i.e. 
    // using FreeType or RustType. Often times this will come as a side-effect 
    // of your map rendering / layouting program.
    //
    // The position is relative to the top left of the grid.
    grid.insert_street(StreetNameRect {
        street_name: String::from("Canterbury Road"),
        x_from_left: Millimeter(30.0),
        width: Millimeter(50.0),
        y_from_top: Millimeter(30.0),
        height: Millimeter(8.0),
    });

    // We deduplicate the roads, i.e.:
    //
    // ```
    // "Canterbury Road" => A3
    // "Canterbury Road" => A4
    // ```
    // 
    // becomes:
    // 
    // ```
    // "Canterbury Road" => [A3, A4]
    // ```
    let deduplicated = DeduplicatedRoads::from_streets(&grid.street_names());

    // As described above, we get both processed and unprocessed 
    // road names back
    let (processed, unprocessed) = deduplicated.process();

    // In this case, "Canterbury Road" spans from B1-B2, so we get a 
    // `ProcessedRoad` back, delimited by a TAB character.
    // 
 	// You can then write this to a CSV file if you want.
    println!("processed:\r\n{}", processed.to_csv("\t"));
    println!("unprocessed:\r\n{}", unprocessed.to_csv("\t"));
}
```

## License

This library is licensed under the MIT license.