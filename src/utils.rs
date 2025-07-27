/// RGB format for colours
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn new_u8(r: u8, b: u8, g: u8) -> Colour {
        Self { r, g, b }
    }

    #[allow(dead_code)]
    pub fn new_f32(r: f32, g: f32, b: f32) -> Colour {
        let ir = (255.999 * r) as u8;
        let ig = (255.999 * g) as u8;
        let ib = (255.999 * b) as u8;
        Self {
            r: ir,
            g: ig,
            b: ib,
        }
    }

    #[allow(dead_code)]
    pub fn to_array(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

pub enum DefinedColours {
    #[allow(dead_code)]
    Red,
    #[allow(dead_code)]
    Blue,
    #[allow(dead_code)]
    Green,
    White,
    Black,
}

impl DefinedColours {
    /// Fetches the [`Colour`] struct value of that DefinedColour
    pub fn colour(&self) -> Colour {
        match self {
            DefinedColours::Red => Colour::new_u8(255, 0, 0),
            DefinedColours::Blue => Colour::new_u8(0, 255, 0),
            DefinedColours::Green => Colour::new_u8(0, 0, 255),
            DefinedColours::White => Colour::new_u8(255, 255, 255),
            DefinedColours::Black => Colour::new_u8(0, 0, 0),
        }
    }
}
