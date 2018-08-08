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

    const ALPHABET_LEN: usize = 26;
    // usize::MAX is "GKGWBYLWRXTLPP" with a length of 15 characters
    const MAX_LEN: usize = 15;
    
    let mut result = [0;MAX_LEN];

    // How many times does 26 fit in the target number?
    let mut multiple_of_alphabet = num / ALPHABET_LEN;
    let mut counter = 0;
    
    while multiple_of_alphabet != 0 && counter < MAX_LEN {
        let remainder = (multiple_of_alphabet - 1) % ALPHABET_LEN;
        result[(MAX_LEN - 1) - counter] = u8_to_char(remainder as u8);
        counter += 1;
        multiple_of_alphabet = (multiple_of_alphabet - 1) / ALPHABET_LEN;
    }

    let len = (MAX_LEN - 1).saturating_sub(counter);
    // Reverse the current characters
    let mut result = result[len..MAX_LEN].iter().map(|c| *c as char).collect::<String>();

    // Push the last character
    result.push(u8_to_char((num % ALPHABET_LEN) as u8) as char);
    
    result
}

#[inline(always)]
fn u8_to_char(input: u8) -> u8 {
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
