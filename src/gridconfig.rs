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

const NUM_MIN: u8 = 'A' as u8;
const NUM_MAX: u8 = 'Z' as u8;
const CHAR_DIFF: u8 = NUM_MAX - NUM_MIN + 1;

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

    let mut result = String::new();

    // input: 6
    // expected output: G (7th letter of the alphabet)
    //
    // calculation:
    // real = 0 since 6.0 / 25.0 = 0.24 and 0.24.floor() = 0
    // rem = 6
    let mut real = (num as f32 / CHAR_DIFF as f32).floor() as usize;
    let mut rem = real % CHAR_DIFF as usize;

    println!("real is initially: {}", real);
    println!("rem is initially: {}", rem);

    while real != 0 {
        // ZA: num = 675 = real = 26
        // 26 % 26 = 0
        result.push(u8_to_char((rem - 1) as u8));
        rem = real % CHAR_DIFF as usize;
        real = (real as f32 / CHAR_DIFF as f32).floor() as usize;
        println!("real is now: {}", real);
    }

    let final_rem = (num - real * CHAR_DIFF as usize) % CHAR_DIFF as usize;
    result.push(u8_to_char(final_rem as u8));

    result
}

fn u8_to_char(input: u8) -> char {
    (NUM_MIN + input) as char
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