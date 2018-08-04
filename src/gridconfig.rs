use roads2csv::{InputStreetValue, StreetName, GridPosition};

#[derive(Debug, Clone)]
pub struct Grid {
    pub bbox: Bbox,
    pub config: GridConfig,
    fonts: Vec<InputStreetValue>,
}

#[derive(Debug, Copy, Clone)]
pub struct GridConfig {
    pub rows: usize,
    pub columns: usize,
    pub row_direction: Ordering,
    pub column_direction: Ordering,
    pub row_offset: usize,
    pub column_offset: usize,
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


        self.fonts.push(InputStreetValue {
            street_name: StreetName(rect.street_name),
            position: GridPosition {
                column: String::from("A"),
                row: 4,
            }
        });
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
    // Ex. input: 35 (AZ)
    // 
    // .. in that case multiple_of_alphabet will be 1, meaning A
    // 
    // 
    let mut multiple_of_alphabet = num / ALPHABET_LEN;

    while multiple_of_alphabet != 0 {
        // 
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

}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Ordering {
    RightToLeft,
    LeftToRight,
}

#[derive(Debug, Copy, Clone)]
pub struct Millimeter(pub f32);

#[derive(Debug, Copy, Clone)]
pub struct Bbox {
    pub width: Millimeter,
    pub height: Millimeter,
}