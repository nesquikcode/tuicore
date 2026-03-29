use std::any::Any;

// Positions / Sizes
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16
}

#[derive(Copy, Clone)]
pub struct RelPosition {
    pub relx: u8,
    pub rely: u8
}

#[derive(Copy, Clone)]
pub enum Positions {
    Position(Position),
    RelPosition(RelPosition)
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16
}

pub struct RelSize {
    pub relwidth: u8,
    pub relheight: u8
}

pub enum Sizes {
    Size(Size),
    RelSize(RelSize)
}

// Impl for relative sizes
impl RelPosition {
    pub fn calc(&self, size: Size) -> Position {
        Position {
            x: ((self.relx as f32 / 100.0) * (size.width as f32)).round() as u16,
            y: ((self.rely as f32 / 100.0) * (size.height as f32)).round() as u16,
        }
    }
}

impl RelSize {
    pub fn calc(&self, size: Size) -> Size {
        Size {
            width: ((self.relwidth as f32 / 100.0) * (size.width as f32)).round() as u16,
            height: ((self.relheight as f32 / 100.0) * (size.height as f32)).round() as u16,
        }
    }
}

impl Sizes {
    pub fn calc(&self, size: Size) -> Size {
        match self {
            Sizes::Size(x) => {
                *x
            }
            Sizes::RelSize(x) => {
                x.calc(size)
            }
        }
    }
}

impl Positions {
    pub fn calc(&self, size: Size) -> Position {
        match self {
            Positions::Position(x) => {
                *x
            },
            Positions::RelPosition(x) => {
                x.calc(size)
            }
        }
    }
}

// Color
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { red: r, green: g, blue: b}
    }
    pub fn maskmax(&self, mask: Color) -> Color {
        let mut res = self.clone();
        if res.red > mask.red {res.red = mask.red.clone()};
        if res.green > mask.green {res.green = mask.green.clone()};
        if res.blue > mask.blue {res.blue = mask.blue.clone()};
        res
    }
    pub fn maskmin(&self, mask: Color) -> Color {
        let mut res = self.clone();
        if res.red < mask.red {res.red = mask.red.clone()};
        if res.green < mask.green {res.green = mask.green.clone()};
        if res.blue < mask.blue {res.blue = mask.blue.clone()};
        res
    }
    pub fn blend_up(&mut self, mut with: Color, opacity: f32) {
        self.red = (self.red as f32 * (1.0 - opacity) + with.red as f32 * opacity) as u8;
        self.green = (self.green as f32 * (1.0 - opacity) + with.green as f32 * opacity) as u8;
        self.blue = (self.blue as f32 * (1.0 - opacity) + with.blue as f32 * opacity) as u8;
    }
    pub fn to_bg_string(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.red, self.green, self.blue)
    }
    pub fn to_fg_string(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue)
    }
    pub fn black() -> Color {
        Color { red: 0, green: 0, blue: 0}
    }
    pub fn white() -> Color {
        Color { red: 255, green: 255, blue: 255}
    }
}

// Pixel
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Pixel {
    pub foreground_color: Color,
    pub background_color: Color,
    pub char: char
}

impl Pixel {
    pub fn new(fg: Option<Color>, bg: Option<Color>, char: Option<char>) -> Pixel {
        let fgcolor: Color = fg.unwrap_or_else(Color::white);
        let bgcolor: Color = bg.unwrap_or_else(Color::black);
        let char: char = match char { Some(x) => x, None => ' ' };
        Pixel { foreground_color: fgcolor, background_color: bgcolor, char: char}
    }
    pub fn empty() -> Pixel {Pixel::new(None, None, None)}
}

// FrameBuffer
pub struct FrameBuffer {
    pub buffer: Vec<Pixel>,
    pub size: Size,
    pub empty: Pixel
}

impl FrameBuffer {
    pub fn new(size: Size, empty: Option<Pixel>) -> FrameBuffer {
        let e = empty.unwrap_or_else(Pixel::empty);
        FrameBuffer {
            buffer: vec![
                e; (size.width as usize) * (size.height as usize)
            ],
            size,
            empty: e
        }
    }
    pub fn init(size: Size) -> FrameBuffer {FrameBuffer::new(size, None)}
    
    pub fn clear(&mut self) {
        self.buffer.fill(self.empty);
    }
    pub fn resize(&mut self, size: Size) {
        let buffsize = self.size.width * self.size.height;
        let screensize = size.width * size.height;
        if buffsize < screensize {
            while ((self.buffer.len() as u16) < screensize) {
                self.buffer.push(self.empty);
            }
            self.size = size;
        } else if buffsize > screensize {
            self.size = size;
            self.buffer.truncate(screensize as usize);
        }
    }
}

// Base element
pub trait Element: Any {
    fn process(&mut self, buff: &mut FrameBuffer);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}