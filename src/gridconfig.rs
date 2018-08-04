use roads2csv::InputStreetValue;

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

impl Grid {

    pub fn new(bbox: Bbox, config: GridConfig) -> Self {
        Self {
            bbox,
            config,
            fonts: Vec::new(),
        }
    }

    pub fn insert_font(rect: StreetNameRect) {
        // TODO!
    }

    pub fn into_street_names(self) -> Vec<InputStreetValue> {
        self.fonts
    }
}


#[derive(Debug, Clone)]
pub struct StreetNameRect {
    pub street_name: String,
    pub x_from_left: Millimeter,
    pub y_from_top: Millimeter,
    pub width: Millimeter,
    pub height: Millimeter,
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

// We have a bounding box with width and height millimeter