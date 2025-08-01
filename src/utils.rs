/// RGB format for colours
#[derive(Clone, Copy, Debug)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Default for Colour {
    fn default() -> Self {
        // The default is the black colour
        DefinedColours::Black.colour()
    }
}

impl Colour {
    #[allow(dead_code)]
    #[deprecated(note = "Use the ::from() function instead")]
    pub fn new_u8(r: u8, b: u8, g: u8) -> Colour {
        Self { r, g, b }
    }

    #[allow(dead_code)]
    #[deprecated(note = "Use the ::from() function instead")]
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
    /// Note: Consider using the ::into() function instead.
    pub fn to_array(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl Into<[u8; 3]> for Colour {
    fn into(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<(u8, u8, u8)> for Colour {
    fn from(value: (u8, u8, u8)) -> Self {
        Self { r: value.0, g: value.1, b: value.2 }
    }
}

impl From<(f32, f32, f32)> for Colour {
    fn from(value: (f32, f32, f32)) -> Self {
        let ir = (255.999 * value.0) as u8;
        let ig = (255.999 * value.1) as u8;
        let ib = (255.999 * value.2) as u8;
        Self {
            r: ir,
            g: ig,
            b: ib,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DefinedColours {
    #[allow(dead_code)]
    Red,
    #[allow(dead_code)]
    Blue,
    #[allow(dead_code)]
    Green,
    #[allow(dead_code)]
    White,
    Black,
}

impl DefinedColours {
    /// Fetches the [`Colour`] struct value of that DefinedColour
    pub fn colour(&self) -> Colour {
        match self {
            DefinedColours::Red => Colour::from((255, 0, 0)),
            DefinedColours::Blue => Colour::from((0, 255, 0)),
            DefinedColours::Green => Colour::from((0, 0, 255)),
            DefinedColours::White => Colour::from((255, 255, 255)),
            DefinedColours::Black => Colour::from((0, 0, 0)),
        }
    }
}
