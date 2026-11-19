use roads2csv::{InputStreetValue, StreetName, GridPosition};

/// The Grid is your street-name grid. Right now there is
/// no support for curved / rotated / translated grids.
///
/// Usually you'll want to initialize this from the page
/// boundaries of your final map, i.e. if you have a map that
/// is (on paper) 290 x 210 mm wide.
///
/// The `config` is for future use to be extended - right now
/// it only stores how big the cells should be. In normal
/// cartography, grids are usually 5 x 5 centimeters (i.e. 50 x 50 mm).
#[derive(Debug, Clone)]
pub struct Grid {
    pub bbox: Bbox,
    pub config: GridConfig,
    fonts: Vec<InputStreetValue>,
    lines: Vec<StreetLine>,
}

/// Unit struct just so it's easier to read that certain values
/// should be in millimeter scale.
#[derive(Debug, Copy, Clone)]
pub struct Millimeter(pub f32);

/// Bounding box (usually the page extents)
#[derive(Debug, Copy, Clone)]
pub struct Bbox {
    pub width: Millimeter,
    pub height: Millimeter,
}

/// Later on this struct will be extended with parameters for
/// curving, rotations, offsets, labeling, etc. Right now
/// it's just: how big should one cell be?
///
/// Cells start at the top left (again, later on this will
/// likely be configurable although I haven't seen a map where
// the grid didn't start at the top left).
#[derive(Debug, Copy, Clone)]
pub struct GridConfig {
    pub cell_height: Millimeter,
    pub cell_width: Millimeter,
}

/// The course of one street on the map: where its road runs, as a line (a polyline in
/// millimeters from the top left of the page). The grid has it for names in three or more grid
/// squares: `DeduplicatedRoads::resolve` finds out from the lines which squares belong to one
/// road and which to another road of the same name. Unlike a `StreetNameRect` this is the road
/// itself, not its label.
#[derive(Debug, Clone, PartialEq)]
pub struct StreetLine {
    pub street_name: StreetName,
    pub points: Vec<(f32, f32)>,
}

/// Represents one street name, layouted on the map. The `StreetNameRect`
/// should be the extent of the font, not of the road itself, because
/// if someone is searching for a road on a map, he will usually scan for the
/// name of the road, not the line of the road. So inserting the extents
/// of the actual road line could lead to problems.
///
/// Calculating these extents should be done via RustType / FreeType or
/// similar, depending on what font you choose. The bounding box should be the
/// minimumn bounding rectangle of the characters that make up the street name.
///
/// For cartographic projections, you have to project the
/// fonts into this coordinate space before adding them, obviously.
/// `street_index` does not take care of any geographic reprojections.
#[derive(Debug, Clone)]
pub struct StreetNameRect {
    pub street_name: String,
    pub x_from_left: Millimeter,
    pub y_from_top: Millimeter,
    pub width: Millimeter,
    pub height: Millimeter,
}


impl Grid {

    /// Initializes an empty grid from a bounding box + configuration
    pub fn new(bbox: Bbox, config: GridConfig) -> Self {
        Self {
            bbox,
            config,
            fonts: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// How many columns and rows the page has: its size over the size of a cell, rounded up
    pub fn size(&self) -> (usize, usize) {
        let columns = (self.bbox.width.0 / self.config.cell_width.0).ceil();
        let rows = (self.bbox.height.0 / self.config.cell_height.0).ceil();
        (columns.max(1.0) as usize, rows.max(1.0) as usize)
    }

    /// Inserts a street and assigns a `GridPosition` (such as "A2" or "B4") to the
    /// road. Note that a `StreetNameRect` may span more than one rectangle, in which
    /// case the road name will be duplicated
    ///
    /// Only the part of the rectangle on the page counts: a name that lies beyond the page
    /// is not inserted (it used to land in row 1 or in a column beyond the grid), and one that
    /// ends exactly on the page's edge does not get a column or row of its own.
    pub fn insert_street(&mut self, rect: StreetNameRect) {

        // the part on the page; the far edges just inside it, as a rectangle that ends on a
        // grid line does not reach into the next square
        let edge = 0.001;
        let (x0, y0) = (rect.x_from_left.0.max(0.0), rect.y_from_top.0.max(0.0));
        let x1 = (rect.x_from_left.0 + rect.width.0).min(self.bbox.width.0 - edge);
        let y1 = (rect.y_from_top.0 + rect.height.0).min(self.bbox.height.0 - edge);
        // (also false for a rectangle with a NaN)
        if !(x1 >= x0 && y1 >= y0) {
            return;
        }

        // ignore direction, etc. for now
        let min_position_x = (x0 / self.config.cell_width.0).floor() as usize;
        let max_position_x = (x1 / self.config.cell_width.0).floor() as usize;

        let mut min_position_y = (y0 / self.config.cell_height.0).floor() as usize;
        let mut max_position_y = (y1 / self.config.cell_height.0).floor() as usize;

        // Y positions have to be adjusted by 1
        // We don't want maps to start at row 0, but rather at row 1
        min_position_y += 1;
        max_position_y += 1;

        let positions_to_add = match (min_position_x == max_position_x, min_position_y == max_position_y) {
            (true, true) => {
                // Street name is contained within one rectangle
                vec![
                    (number_to_alphabet_value(min_position_x), min_position_y),
                ]
            },
            (true, false) => {
                // Street name is contained within one column
                vec![
                    (number_to_alphabet_value(min_position_x), min_position_y),
                    (number_to_alphabet_value(min_position_x), max_position_y),
                ]
            },
            (false, true) => {
                // Street name is contained within one row
                vec![
                    (number_to_alphabet_value(min_position_x), min_position_y),
                    (number_to_alphabet_value(max_position_x), min_position_y),
                ]
            },
            (false, false) => {
                // Street name overlaps 4 quadrants
                vec![
                    (number_to_alphabet_value(min_position_x), min_position_y),
                    (number_to_alphabet_value(min_position_x), max_position_y),
                    (number_to_alphabet_value(max_position_x), min_position_y),
                    (number_to_alphabet_value(max_position_x), max_position_y),
                ]
            }
        };

        for (column, row) in positions_to_add {
            self.fonts.push(InputStreetValue {
                street_name: StreetName(rect.street_name.clone()),
                position: GridPosition {
                    column,
                    row,
                }
            });
        }
    }

    /// Returns all the fonts in the grid that were added previously
    pub fn street_names(&self) -> Vec<InputStreetValue> {
        self.fonts.clone()
    }

    /// Inserts the course of a street (a polyline in millimeters from the top left of the
    /// page): where a road of this name runs. Not needed for names in one or two squares;
    /// `DeduplicatedRoads::resolve` uses the lines for the names in three or more.
    pub fn insert_street_line(&mut self, street_name: &str, points: &[(f32, f32)]) {
        if points.len() >= 2 {
            self.lines.push(StreetLine { street_name: StreetName(street_name.to_string()), points: points.to_vec() });
        }
    }

    /// The street lines in the grid that were added previously
    pub fn street_lines(&self) -> &[StreetLine] {
        &self.lines
    }

    /// The squares a polyline (millimeters from the top left of the page) passes, as
    /// `(column, row)`, counted from 0: those with at least `min_mm` of the line in them; a
    /// line with less in every square: the square most of it lies in. The part of the line
    /// beyond the page does not count.
    pub fn squares_passed(&self, points: &[(f32, f32)], min_mm: f32) -> Vec<(usize, usize)> {
        let (columns, rows) = self.size();
        let (cell_w, cell_h) = (self.config.cell_width.0, self.config.cell_height.0);
        let (w, h) = (self.bbox.width.0, self.bbox.height.0);
        let mut mm: ::std::collections::BTreeMap<(usize, usize), f32> = ::std::collections::BTreeMap::new();
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let len = (b.0 - a.0).hypot(b.1 - a.1);
            if !len.is_finite() || len <= 0.0 {
                continue;
            }
            // a sample every millimeter or less, each counted for its length
            let steps = len.ceil().max(1.0) as usize;
            let step = len / steps as f32;
            for k in 0..steps {
                let t = (k as f32 + 0.5) / steps as f32;
                let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                if x >= 0.0 && y >= 0.0 && x < w && y < h {
                    let cell = (((x / cell_w) as usize).min(columns - 1), ((y / cell_h) as usize).min(rows - 1));
                    *mm.entry(cell).or_insert(0.0) += step;
                }
            }
        }
        let mut squares: Vec<(usize, usize)> = mm.iter().filter(|&(_, v)| *v >= min_mm).map(|(c, _)| *c).collect();
        if squares.is_empty() {
            let mut best: Option<((usize, usize), f32)> = None;
            for (c, v) in mm.iter() {
                if best.map(|b| *v > b.1).unwrap_or(true) {
                    best = Some((*c, *v));
                }
            }
            squares.extend(best.map(|b| b.0));
        }
        squares
    }
}

/// Maps an index number to a value, necessary for creating the street index. i.e.:
///
/// ```no_run,ignore
/// 0   -> A
/// 25  -> Z
/// 26  -> AA
/// 27  -> AB
/// ```
///
/// ... and so on
pub fn number_to_alphabet_value(num: usize) -> String {
    // Maximum character count range is 26 characters, A to Z
    const ALPHABET_LEN: usize = 26;
    // usize::MAX is "GKGWBYLWRXTLPP" with a length of 15 characters
    const MAX_LEN: usize = 15;

    // Initialize an array of 15 characters all to 0
    let mut result = [0;MAX_LEN + 1];

    // ** For this example, assume that we get the number 80000 **

    // How many times does 26 fit in the target number?
    // 80000 / 26 = 3076
    let mut multiple_of_alphabet = num / ALPHABET_LEN;

    // How many characters have we created in the loop?
    let mut character_count = 0;


    // If multiple_of_alphabet is 0, that means that `num` has decreased
    // to a range between 0 and 26.
    //
    // `counter < MAX_LEN` is just so that the optimizer can
    // unroll the loop without bounds-checking.
    while multiple_of_alphabet != 0 && character_count < MAX_LEN {
        // The "remainder" is our target character that we push into the array.
        //
        // For example, if `multiple_of_alphabet` is 3076, that means that
        // num is in the range of (26 * 3076) to (26 * 3077).
        //
        // Therefore, we want to take the remainder of the last place, essentially:
        // (3076 - 1) % 26 = 7 = "H"
        let remainder = (multiple_of_alphabet - 1) % ALPHABET_LEN;
        // Push the "H" into the array
        result[(MAX_LEN - 1) - character_count] = u8_to_char(remainder as u8);
        // We pushed one character, increase the character_count by 1
        character_count += 1;
        // Now we prepare the next character - currently, multiple_of_alphabet is 3075.
        // Integer division always rounds down, which is useful property:
        // Now our array is:
        //
        // [0, 0, 0, ... "H", 0]
        // (3076 - 1) / 26 = 118
        //
        // The next iteration will be:
        //
        // (118 - 1) % 26 = 13 = "N"
        // [0, 0, 0, ... "N", "H", 0]
        // (118 - 1) / 26 = 4
        //
        // (4 - 1) % 26 = 4 = "D"
        // [0, 0, 0, ... "D", "N", "H", 0]
        // (4 - 1) / 26 = 0 = quit the loop
        multiple_of_alphabet = (multiple_of_alphabet - 1) / ALPHABET_LEN;
    }

    // Last character: 80.000 % 26 = 24 = "Y"
    // [0, 0, 0, ... "D", "N", "H", "Y"]
    result[MAX_LEN] = u8_to_char((num % ALPHABET_LEN) as u8);

    // count is 3, since we pushed 3 characters
    // zeroed_characters will be the offset from the start of the array
    //
    // so: 15 - 3 = 12, to take the characters from 12 to 16 (the array is [MAX_LEN + 1]).
    //
    // zeroed_characters is the number of characters that are still set to 0
    // in the result array. We want to ignore all characters that are set to 0.
    //
    // Note that this is MAX_LEN, not MAX_LEN + 1
    let zeroed_characters = MAX_LEN.saturating_sub(character_count);

    // We take a slice from the zeroed_characters to the end of the array
    // (i.e. MAX_LEN + 1). Note that we have to include the final character.
    let slice = unsafe { result.get_unchecked(zeroed_characters..) };

    // Cast the slice to a string, since we know that we only have ASCII
    // characters in the range from A to Z, there won't be any UTF-8 problems
    unsafe { ::std::str::from_utf8_unchecked(slice) }.to_string()
}

/// The index of a column's letters, the inverse of `number_to_alphabet_value`:
///
/// ```no_run,ignore
/// "A"  -> Some(0)
/// "Z"  -> Some(25)
/// "AA" -> Some(26)
/// "a"  -> None
/// ```
pub fn alphabet_value_to_number(letters: &str) -> Option<usize> {
    if letters.is_empty() || !letters.bytes().all(|b| b >= b'A' && b <= b'Z') {
        return None;
    }
    let mut value: usize = 0;
    for b in letters.bytes() {
        value = match value.checked_mul(26).and_then(|v| v.checked_add((b - b'A') as usize + 1)) {
            Some(v) => v,
            None => return None,
        };
    }
    Some(value - 1)
}

// Transform from 0 to A, 1 to B, etc.
#[inline(always)]
fn u8_to_char(input: u8) -> u8 {
    // use 'a' as u8 to create lowercase characters
    'A' as u8 + input
}

#[test]
fn test_number_to_alphabet_value() {
    assert_eq!(number_to_alphabet_value(0), String::from("A"));
    assert_eq!(number_to_alphabet_value(1), String::from("B"));
    assert_eq!(number_to_alphabet_value(6), String::from("G"));
    assert_eq!(number_to_alphabet_value(26), String::from("AA"));
    assert_eq!(number_to_alphabet_value(27), String::from("AB"));
    assert_eq!(number_to_alphabet_value(225), String::from("HR"));
}

#[cfg(all(test, feature = "nightly"))]
mod tests {
    use super::*;
    use test::Bencher;
    use test;

    #[bench]
    fn bench_number_to_alphabet_value(b: &mut Bencher) {

        b.iter(|| {
            let n = test::black_box(usize::max_value());
            number_to_alphabet_value(n);
        });
    }
}