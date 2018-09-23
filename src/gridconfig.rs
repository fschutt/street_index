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
        }
    }

    /// Inserts a street and assigns a `GridPosition` (such as "A2" or "B4") to the
    /// road. Note that a `StreetNameRect` may span more than one rectangle, in which
    /// case the road name will be duplicated
    pub fn insert_street(&mut self, rect: StreetNameRect) {

        // ignore direction, etc. for now
        let min_position_x = (rect.x_from_left.0 / self.config.cell_width.0).floor() as usize;
        let max_position_x = ((rect.x_from_left.0 + rect.width.0) / self.config.cell_width.0).floor() as usize;

        let mut min_position_y = (rect.y_from_top.0 / self.config.cell_height.0).floor() as usize;
        let mut max_position_y = ((rect.y_from_top.0 + rect.height.0) / self.config.cell_height.0).floor() as usize;

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
    let slice = &result[zeroed_characters..];

    // Cast the slice to a string, since we know that we only have ASCII
    // characters in the range from A to Z, there won't be any UTF-8 problems
    unsafe { ::std::str::from_utf8_unchecked(slice) }.to_string()
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