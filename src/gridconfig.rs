use roads2csv::{InputStreetValue, StreetName, GridPosition};

#[derive(Debug, Clone)]
pub struct Grid {
    pub bbox: Bbox,
    pub config: GridConfig,
    fonts: Vec<InputStreetValue>,
}

#[derive(Debug, Copy, Clone)]
pub struct Millimeter(pub f32);

#[derive(Debug, Copy, Clone)]
pub struct Bbox {
    pub width: Millimeter,
    pub height: Millimeter,
}

#[derive(Debug, Copy, Clone)]
pub struct GridConfig {
    pub cell_height: Millimeter,
    pub cell_width: Millimeter,
}

#[derive(Debug, Clone)]
pub struct StreetNameRect {
    pub street_name: String,
    pub x_from_left: Millimeter,
    pub y_from_top: Millimeter,
    pub width: Millimeter,
    pub height: Millimeter,
}


impl Grid {

    pub fn new(bbox: Bbox, config: GridConfig) -> Self {
        Self {
            bbox,
            config,
            fonts: Vec::new(),
        }
    }

    pub fn insert_font(&mut self, rect: StreetNameRect) {

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

    pub fn into_street_names(self) -> Vec<InputStreetValue> {
        self.fonts
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

    let mut result = Vec::<char>::new();

    // How many times does 26 fit in the target number?
    let mut multiple_of_alphabet = num / ALPHABET_LEN;

    while multiple_of_alphabet != 0 {
        let remainder = (multiple_of_alphabet - 1) % ALPHABET_LEN;
        result.push(u8_to_char(remainder as u8));
        multiple_of_alphabet = (multiple_of_alphabet - 1) / ALPHABET_LEN;
    }

    // Reverse the current characters
    let mut result = result.into_iter().rev().collect::<String>();

    // Push the last characters
    result.push(u8_to_char((num % ALPHABET_LEN) as u8));
    
    result

}

fn u8_to_char(input: u8) -> char {
    ('A' as u8 + input) as char
}

#[test]
fn u8_to_char_test() {
    assert_eq!(u8_to_char(0), 'A');
    assert_eq!(u8_to_char(6), 'G');
    assert_eq!(u8_to_char(25), 'Z');
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
