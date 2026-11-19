//! Resolves the names in three or more grid squares with the course of their streets
//!
//! `DeduplicatedRoads::process` prints a name in one or two grid squares ("A9", "A9-B2") and
//! hands everything else back for manual review: from the squares alone one cannot tell two roads
//! of the same name (`A9-B10; E1-E2`) from one long road that is partly clipped off the map
//! (`A9-E2`). With the course of each street (`Grid::insert_street_line`) the grid can tell: per
//! connected street of the name, the squares it passes make one range; ranges over or next to
//! each other are one range (a name across a corner of the grid lies in four squares, and a
//! reader searches them as one).
//!
//! ```
//! use street_index::prelude::*;
//!
//! let mut grid = Grid::new(
//!     Bbox { width: Millimeter(200.0), height: Millimeter(100.0) },
//!     GridConfig { cell_width: Millimeter(50.0), cell_height: Millimeter(50.0) });
//! // "Nordring" is labelled in three places along one road across the page
//! for x in &[10.0, 85.0, 160.0] {
//!     grid.insert_street(StreetNameRect {
//!         street_name: "Nordring".to_string(),
//!         x_from_left: Millimeter(*x), width: Millimeter(30.0),
//!         y_from_top: Millimeter(20.0), height: Millimeter(3.0),
//!     });
//! }
//! grid.insert_street_line("Nordring", &[(5.0, 21.0), (195.0, 21.0)]);
//! let roads = DeduplicatedRoads::from_streets(&grid.street_names()).resolve(&grid);
//! // `process()` would have given "Nordring" back for review: it is in A1, B1 and D1
//! assert_eq!(roads.roads[0].position(), "A1-D1");
//! ```

use std::fmt;

use gridconfig::{alphabet_value_to_number, number_to_alphabet_value, Grid};
use roads2csv::{DeduplicatedRoads, StreetName};

/// A road passes a grid square when this much of it (millimeters) lies in the square
pub const ROAD_IN_SQUARE_MM: f32 = 5.0;

/// A range of grid squares a reader searches: columns `first_column..=last_column`, rows
/// `first_row..=last_row`, counted from 0 (the row printed is one more: "A1" is column 0, row 0)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GridRange {
    pub first_column: usize,
    pub first_row: usize,
    pub last_column: usize,
    pub last_row: usize,
}

impl GridRange {
    /// The range around these squares `(column, row)`; `None` for no squares
    pub fn around(squares: &[(usize, usize)]) -> Option<GridRange> {
        let first = match squares.first() {
            Some(s) => *s,
            None => return None,
        };
        let mut range = GridRange { first_column: first.0, first_row: first.1, last_column: first.0, last_row: first.1 };
        for s in squares.iter() {
            range = range.union(&GridRange { first_column: s.0, first_row: s.1, last_column: s.0, last_row: s.1 });
        }
        Some(range)
    }

    /// The range around both
    pub fn union(&self, other: &GridRange) -> GridRange {
        GridRange {
            first_column: self.first_column.min(other.first_column),
            first_row: self.first_row.min(other.first_row),
            last_column: self.last_column.max(other.last_column),
            last_row: self.last_row.max(other.last_row),
        }
    }

    /// Over each other, or next to each other (sides and corners)
    pub fn touches(&self, other: &GridRange) -> bool {
        self.first_column <= other.last_column + 1 && other.first_column <= self.last_column + 1
            && self.first_row <= other.last_row + 1 && other.first_row <= self.last_row + 1
    }

    /// Whether the square `(column, row)` is in the range
    pub fn contains(&self, square: (usize, usize)) -> bool {
        square.0 >= self.first_column && square.0 <= self.last_column && square.1 >= self.first_row && square.1 <= self.last_row
    }

    /// How many squares are in the range
    pub fn squares(&self) -> usize {
        (self.last_column - self.first_column + 1) * (self.last_row - self.first_row + 1)
    }
}

/// "B6" for one square, "A1-C4" for more (top left to bottom right)
impl fmt::Display for GridRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let from = format!("{}{}", number_to_alphabet_value(self.first_column), self.first_row + 1);
        if self.first_column == self.last_column && self.first_row == self.last_row {
            write!(f, "{}", from)
        } else {
            write!(f, "{}-{}{}", from, number_to_alphabet_value(self.last_column), self.last_row + 1)
        }
    }
}

/// A street name with where a reader finds it
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoad {
    pub name: StreetName,
    /// the squares `(column, row)` the name's labels lie in, counted from 0
    pub squares: Vec<(usize, usize)>,
    /// the ranges to search: one for a name in one or two squares (as `process()` prints it),
    /// else one per street of the name
    pub ranges: Vec<GridRange>,
    /// made with the courses of the streets (a name in three or more squares)
    pub resolved: bool,
    /// how many separate streets the ranges are made of (1 for a name in one or two squares)
    pub streets: usize,
}

impl ResolvedRoad {
    /// "A9", "A9-B2" or the ranges, "A1-C4, F6"
    pub fn position(&self) -> String {
        self.ranges.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ")
    }
}

impl fmt::Display for ResolvedRoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.name, self.position())
    }
}

/// Every name with where a reader finds it, sorted by name
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolvedRoads {
    pub roads: Vec<ResolvedRoad>,
}

impl ResolvedRoads {
    pub fn to_csv(&self, delimiter: &str) -> String {
        self.roads.iter().map(|r| format!("{}{}{}", r.name, delimiter, r.position())).collect::<Vec<_>>().join("\r\n")
    }
}

impl DeduplicatedRoads {
    /// Like `process()`, but a name in three or more grid squares is not handed back: it gets
    /// the ranges of its streets, made from the lines of the grid (`Grid::insert_street_line`).
    /// A name in one square, or in two next to each other, is as `process()` has it. A name in
    /// two squares apart (`process()` prints "B1-F3", a range of 15 squares) is resolved like one
    /// in three. A name with no line of its own is listed by the squares of its labels (each
    /// square a range, ranges that touch one range).
    pub fn resolve(&self, grid: &Grid) -> ResolvedRoads {
        let (columns, rows) = grid.size();
        let mut roads = Vec::new();
        for (name, positions) in self.roads.iter() {
            let squares: Vec<(usize, usize)> = positions.iter()
                .filter_map(|p| alphabet_value_to_number(&p.column).and_then(|c| p.row.checked_sub(1).map(|r| (c, r))))
                .filter(|&(c, r)| c < columns && r < rows)
                .collect();
            let whole = match GridRange::around(&squares) {
                Some(r) => r,
                None => continue,
            };
            // one square, or two next to each other (a label across a grid line): as `process()`
            // prints it; two squares apart (two labels of the name, or a road between them) are
            // as ambiguous as three
            let together = squares.len() == 1
                || (squares.len() == 2 && GridRange::around(&squares[..1]).unwrap().touches(&GridRange::around(&squares[1..]).unwrap()));
            if together {
                roads.push(ResolvedRoad { name: name.clone(), squares, ranges: vec![whole], resolved: false, streets: 1 });
                continue;
            }
            let lines: Vec<&[(f32, f32)]> = grid.street_lines().iter().filter(|l| l.street_name == *name).map(|l| &l.points[..]).collect();
            let (ranges, streets) = resolve_ranges(grid, &squares, &lines);
            roads.push(ResolvedRoad { name: name.clone(), squares, ranges, resolved: true, streets });
        }
        ResolvedRoads { roads }
    }
}

/// The ranges a reader searches a name in: per street (line) that passes a square the name's
/// labels lie in, the range around the squares it passes; the labels' squares no street passes
/// stay on their own; ranges that touch are one range, from the left. And how many streets
fn resolve_ranges(grid: &Grid, labels: &[(usize, usize)], lines: &[&[(f32, f32)]]) -> (Vec<GridRange>, usize) {
    let mut parts: Vec<GridRange> = Vec::new();
    let mut covered: Vec<(usize, usize)> = Vec::new();
    let mut streets = 0;
    for line in lines.iter() {
        let passed = grid.squares_passed(line, ROAD_IN_SQUARE_MM);
        if !passed.iter().any(|s| labels.contains(s)) {
            continue;
        }
        streets += 1;
        covered.extend(passed.iter().cloned());
        parts.extend(GridRange::around(&passed));
    }
    for s in labels.iter().filter(|s| !covered.contains(s)) {
        parts.extend(GridRange::around(&[*s]));
    }
    let mut merged: Vec<GridRange> = Vec::new();
    for part in parts {
        merged.push(part);
        loop {
            let mut pair = None;
            'search: for a in 0..merged.len() {
                for b in a + 1..merged.len() {
                    if merged[a].touches(&merged[b]) {
                        pair = Some((a, b));
                        break 'search;
                    }
                }
            }
            match pair {
                Some((a, b)) => {
                    let union = merged[a].union(&merged[b]);
                    merged.remove(b);
                    merged[a] = union;
                }
                None => break,
            }
        }
    }
    merged.sort_by_key(|r| (r.first_column, r.first_row, r.last_column, r.last_row));
    (merged, ::std::cmp::max(streets, 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridconfig::{Bbox, GridConfig, Millimeter, StreetNameRect};

    /// 240 x 120 mm in squares of 40 mm: A1 to F3
    fn grid() -> Grid {
        Grid::new(
            Bbox { width: Millimeter(240.0), height: Millimeter(120.0) },
            GridConfig { cell_width: Millimeter(40.0), cell_height: Millimeter(40.0) },
        )
    }

    fn label(grid: &mut Grid, name: &str, x: f32, y: f32, w: f32) {
        grid.insert_street(StreetNameRect {
            street_name: name.to_string(),
            x_from_left: Millimeter(x), width: Millimeter(w),
            y_from_top: Millimeter(y), height: Millimeter(3.0),
        });
    }

    fn text(roads: &ResolvedRoads, name: &str) -> String {
        roads.roads.iter().find(|r| r.name.0 == name).map(|r| r.position()).unwrap_or_else(|| "(none)".to_string())
    }

    #[test]
    fn ranges_read_as_a_street_index_prints_them() {
        assert_eq!(GridRange::around(&[(0, 0), (2, 3)]).unwrap().to_string(), "A1-C4");
        assert_eq!(GridRange::around(&[(5, 1)]).unwrap().to_string(), "F2");
        assert_eq!(GridRange::around(&[(26, 9)]).unwrap().to_string(), "AA10");
        assert_eq!(GridRange::around(&[]), None);
        let r = GridRange::around(&[(1, 1), (3, 2)]).unwrap();
        assert_eq!(r.squares(), 6);
        assert!(r.contains((2, 1)) && !r.contains((0, 1)));
        assert!(r.touches(&GridRange::around(&[(4, 3)]).unwrap()) && !r.touches(&GridRange::around(&[(5, 3)]).unwrap()));
    }

    #[test]
    fn columns_are_read_back_from_their_letters() {
        for n in &[0usize, 1, 25, 26, 27, 51, 52, 701, 702, 18277] {
            assert_eq!(alphabet_value_to_number(&number_to_alphabet_value(*n)), Some(*n), "{}", n);
        }
        assert_eq!(alphabet_value_to_number("a"), None);
        assert_eq!(alphabet_value_to_number(""), None);
        assert_eq!(alphabet_value_to_number("A1"), None);
    }

    #[test]
    fn a_road_passes_the_squares_with_5_mm_of_it() {
        let g = grid();
        // along y = 20 from x = 10 to x = 130: A1 (30 mm), B1 (40), C1 (40), D1 (10)
        assert_eq!(g.squares_passed(&[(10.0, 20.0), (130.0, 20.0)], ROAD_IN_SQUARE_MM), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
        // 3 mm in the next square: not passed
        assert_eq!(g.squares_passed(&[(10.0, 20.0), (123.0, 20.0)], ROAD_IN_SQUARE_MM), vec![(0, 0), (1, 0), (2, 0)]);
        // under 5 mm in every square: the one most of it lies in
        assert_eq!(g.squares_passed(&[(38.0, 10.0), (43.0, 10.0)], ROAD_IN_SQUARE_MM), vec![(1, 0)]);
        // the part beyond the page does not count
        assert_eq!(g.squares_passed(&[(220.0, 20.0), (300.0, 20.0)], ROAD_IN_SQUARE_MM), vec![(5, 0)]);
        assert!(g.squares_passed(&[(300.0, 20.0), (320.0, 20.0)], ROAD_IN_SQUARE_MM).is_empty());
    }

    #[test]
    fn a_name_beyond_the_page_is_not_listed() {
        let mut g = grid();
        label(&mut g, "Oben", 20.0, -10.0, 15.0);
        label(&mut g, "Rechts", 250.0, 20.0, 15.0);
        label(&mut g, "Unten", 20.0, 130.0, 15.0);
        label(&mut g, "Drinnen", 20.0, 20.0, 15.0);
        // across the right edge: only its square on the page; ending on the edge: no column of its own
        label(&mut g, "Rand", 232.0, 20.0, 15.0);
        label(&mut g, "Kante", 225.0, 20.0, 15.0);
        let roads = DeduplicatedRoads::from_streets(&g.street_names()).resolve(&g);
        assert_eq!(roads.roads.iter().map(|r| (r.name.0.as_str(), r.position())).collect::<Vec<_>>(),
            vec![("Drinnen", "A1".to_string()), ("Kante", "F1".to_string()), ("Rand", "F1".to_string())]);
    }

    #[test]
    fn a_name_in_one_or_two_squares_is_as_process_prints_it() {
        let mut g = grid();
        label(&mut g, "Kurz", 10.0, 10.0, 15.0);
        label(&mut g, "Quer", 30.0, 20.0, 20.0);
        let deduplicated = DeduplicatedRoads::from_streets(&g.street_names());
        let resolved = deduplicated.resolve(&g);
        assert_eq!(text(&resolved, "Kurz"), "A1");
        assert_eq!(text(&resolved, "Quer"), "A1-B1");
        let (processed, unprocessed) = deduplicated.process();
        assert_eq!((processed.processed.len(), unprocessed.unprocessed.len()), (2, 0));
        assert_eq!(processed.processed[1].position.to_string(), "A1-B1");
        assert!(resolved.roads.iter().all(|r| !r.resolved));
    }

    #[test]
    fn a_name_in_three_or_more_squares_gets_the_range_of_its_street() {
        let mut g = grid();
        // one road along y = 20, labelled in A1, C1 and E1: `process()` hands it back
        label(&mut g, "Nordring", 10.0, 20.0, 15.0);
        label(&mut g, "Nordring", 90.0, 20.0, 15.0);
        label(&mut g, "Nordring", 170.0, 20.0, 15.0);
        g.insert_street_line("Nordring", &[(5.0, 21.0), (235.0, 21.0)]);
        let deduplicated = DeduplicatedRoads::from_streets(&g.street_names());
        assert_eq!(deduplicated.process().1.unprocessed.len(), 1);
        let roads = deduplicated.resolve(&g);
        let r = &roads.roads[0];
        assert_eq!((r.position().as_str(), r.resolved, r.streets, r.squares.len()), ("A1-F1", true, 1, 3));
        assert_eq!(roads.to_csv("\t"), "Nordring\tA1-F1");
    }

    #[test]
    fn separate_streets_of_one_name_are_separate_ranges() {
        let mut g = grid();
        label(&mut g, "Ring", 50.0, 20.0, 10.0);
        label(&mut g, "Ring", 62.0, 20.0, 10.0);
        label(&mut g, "Ring", 200.0, 100.0, 10.0);
        g.insert_street_line("Ring", &[(5.0, 20.0), (75.0, 20.0)]);
        g.insert_street_line("Ring", &[(130.0, 100.0), (235.0, 100.0)]);
        // a street of the name with no label: not one of its ranges
        g.insert_street_line("Ring", &[(100.0, 60.0), (130.0, 62.0)]);
        // another name's line is not this name's
        g.insert_street_line("Anderswo", &[(0.0, 0.0), (240.0, 0.0)]);
        let roads = DeduplicatedRoads::from_streets(&g.street_names()).resolve(&g);
        assert_eq!((text(&roads, "Ring"), roads.roads[0].streets), ("A1-B1, D3-F3".to_string(), 2));
    }

    #[test]
    fn two_squares_apart_are_not_one_range() {
        let mut g = grid();
        label(&mut g, "Fern", 10.0, 10.0, 15.0);
        label(&mut g, "Fern", 210.0, 100.0, 10.0);
        let deduplicated = DeduplicatedRoads::from_streets(&g.street_names());
        // `process()` prints the pair as a range of 18 squares
        assert_eq!(deduplicated.process().0.processed[0].position.to_string(), "A1-F3");
        assert_eq!(text(&deduplicated.resolve(&g), "Fern"), "A1, F3");
        // a road that passes the squares between them is one range
        g.insert_street_line("Fern", &[(5.0, 20.0), (235.0, 20.0)]);
        g.insert_street_line("Fern", &[(210.0, 20.0), (210.0, 110.0)]);
        assert_eq!(text(&DeduplicatedRoads::from_streets(&g.street_names()).resolve(&g), "Fern"), "A1-F3");
    }

    #[test]
    fn a_name_without_a_line_is_listed_by_its_labels_squares() {
        let mut g = grid();
        // across a corner of the grid: four squares (B1, C1, B2, C2); and one far away
        label(&mut g, "Ecke", 70.0, 38.0, 20.0);
        label(&mut g, "Ecke", 210.0, 100.0, 10.0);
        let roads = DeduplicatedRoads::from_streets(&g.street_names()).resolve(&g);
        assert_eq!(text(&roads, "Ecke"), "B1-C2, F3");
    }
}
