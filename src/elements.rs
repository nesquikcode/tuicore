use std::any::Any;
use crate::base::{
    Position, RelPosition, Positions,
    Size, RelSize, Sizes,
    Color, Pixel, FrameBuffer,
    Element
};
use image::{DynamicImage, GenericImageView, ImageResult, open, imageops::{FilterType}};


// Label
#[derive(Clone)]
pub struct Label {
    pos: Positions,
    content: String,
    fg_color: Color,
    bg_color: Color,
    fg_opacity: f32,
    bg_opacity: f32,
    centered: bool
}

impl Label {
    pub fn new(content: String, pos: Positions, centered: Option<bool>, fg: Option<Color>, bg: Option<Color>, fg_opacity: Option<f32>, bg_opacity: Option<f32>) -> Label {
        Label {
            pos,
            content,
            fg_color: fg.unwrap_or_else(Color::white),
            bg_color: bg.unwrap_or_else(Color::black),
            fg_opacity: fg_opacity.unwrap_or_else(|| 1.0),
            bg_opacity: fg_opacity.unwrap_or_else(|| 0.0),
            centered: centered.unwrap_or_else(|| false)
        }
    }
    pub fn change_content(&mut self, content: String) {self.content = content;}
    pub fn change_pos(&mut self, pos: Positions) {self.pos = pos;}
    pub fn change_fg(&mut self, color: Color) {self.fg_color = color;}
    pub fn change_bg(&mut self, color: Color) {self.bg_color = color;}
}
impl Element for Label {
    fn process(&mut self, buff: &mut FrameBuffer) {
        let pos = self.pos.calc(buff.size.clone());
        let x;
        match self.centered {
            true => {
                x = pos.x - (self.content.len() as u16) / 2;
            }
            false => {
                x = pos.x;
            }
        }
        
        for (i, ch) in self.content.chars().enumerate() {
            let idx = (pos.y*buff.size.width+(x+(i as u16))) as usize;
            let pixel = match buff.buffer.get(idx) { Some(x) => x, None => {continue;}};
            let mut pixelFg = pixel.foreground_color;
            let mut pixelBg = pixel.background_color;
            pixelFg.blend_up(self.fg_color, self.fg_opacity);
            pixelBg.blend_up(self.bg_color, self.bg_opacity);
            buff.buffer[(pos.y*buff.size.width+(x+(i as u16))) as usize] = Pixel::new(
                Some(pixelFg),
                Some(pixelBg),
                Some(ch.clone())
            );
        }
    }
    fn as_any(&self) -> &dyn Any {self}
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

// Rectangle
pub struct Rectangle {
    pos: Positions,
    size: Sizes,
    fg_color: Color,
    bg_color: Color,
    fg_opacity: f32,
    bg_opacity: f32,
    centered: bool
}
impl Rectangle {
    pub fn new(pos: Positions, size: Sizes, bg_color: Option<Color>, fg_color: Option<Color>, fg_opacity: Option<f32>, bg_opacity: Option<f32>, centered: Option<bool>) -> Rectangle {
        Rectangle {
            pos,
            size,
            fg_color: fg_color.unwrap_or_else(Color::white),
            bg_color: bg_color.unwrap_or_else(Color::black),
            fg_opacity: fg_opacity.unwrap_or_else(|| 1.0),
            bg_opacity: bg_opacity.unwrap_or_else(|| 1.0),
            centered: centered.unwrap_or_else(|| false)
        }
    }
    pub fn change_pos(&mut self, pos: Positions) {self.pos = pos;}
    pub fn change_size(&mut self, size: Sizes) {self.size = size;}
    pub fn change_fg(&mut self, color: Color) {self.fg_color = color;}
    pub fn change_bg(&mut self, color: Color) {self.bg_color = color;}
}
impl Element for Rectangle {
    fn process(&mut self, buff: &mut FrameBuffer) {
        let mut pos = self.pos.calc(buff.size);
        let size = self.size.calc(buff.size);
        match self.centered {
            true => {
                pos.x = pos.x - (size.width / 2);
                pos.y = pos.y - (size.height / 2);
            }
            false => {}
        }
        for y in 0..size.height {
            for x in 0..size.width {
                let idx = ((pos.y + y) * buff.size.width + (pos.x + x)) as usize;
                let pixel = match buff.buffer.get(idx) {Some(x) => x, None => {continue;}};
                let mut pixelFg = pixel.foreground_color;
                let mut pixelBg = pixel.background_color;
                pixelFg.blend_up(self.fg_color, self.fg_opacity);
                pixelBg.blend_up(self.bg_color, self.bg_opacity);
                buff.buffer[idx] = Pixel::new(Some(pixelFg), Some(pixelBg), None);
            }
        }
    }
    fn as_any(&self) -> &dyn Any {self}
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

// Image
pub struct ImageCache {
    at_width: u32,
    at_height: u32,
    data: Vec<Pixel>
}
pub struct Image {
    pos: Positions,
    size: Sizes,
    image: DynamicImage,
    cache: ImageCache,
    opacity: f32,
    centered: bool
}
impl Image {
    pub fn new(pos: Positions, size: Sizes, path: String, opacity: Option<f32>, centered: Option<bool>) -> Image{
        let img = open(path).unwrap();
        let mut pixels: Vec<Pixel> = Vec::new();
        for (x, y, c) in img.pixels() {
            pixels.push(
                Pixel::new(None, Some(Color::new(c[0], c[1], c[2])), None)
            )
        }
        Image {
            pos: pos,
            size: size,
            image: img.clone(),
            opacity: opacity.unwrap_or_else(|| 1.0),
            cache: ImageCache {
                at_width: img.width(), at_height: img.height(), data: pixels
            },
            centered: centered.unwrap_or_else(|| false)
        }
    }
    pub fn get_at_size(&mut self, width: u32, height: u32) -> Vec<Pixel> {
        if self.cache.at_width == width && self.cache.at_height == height {
            return self.cache.data.clone();
        };
        let mut pixels: Vec<Pixel> = vec![Pixel::empty(); (width*height) as usize];

        for (x, y, c) in self.image.resize_exact(width, height, FilterType::Nearest).pixels() {
            pixels.insert(
                (y * width + x) as usize,
                Pixel::new(None, Some(Color::new(c[0], c[1], c[2])), None)
            )
        };
        self.cache.data = pixels.clone();
        self.cache.at_width = width;
        self.cache.at_height = height;
        pixels
    }
}
impl Element for Image {
    fn process(&mut self, buff: &mut FrameBuffer) {
        let mut pos = self.pos.calc(buff.size);
        let size = self.size.calc(buff.size);
        let pixels = self.get_at_size(size.width as u32, size.height as u32);
        match self.centered {
            true => {
                pos.x = pos.x - (size.width / 2);
                pos.y = pos.y - (size.height / 2);
            }
            false => {}
        }
        for y in 0..size.height {
            for x in 0..size.width {
                let idx = ((pos.y + y) * buff.size.width + (pos.x + x)) as usize;
                let canvasPixel = match buff.buffer.get(idx) {Some(x) => x, None => {continue;}};
                let imagePixel = match pixels.get((y * size.width + x) as usize) {Some(x) => x, None => {continue;}};
                let mut pixelFg = canvasPixel.foreground_color;
                let mut pixelBg = canvasPixel.background_color;
                pixelFg.blend_up(imagePixel.foreground_color, self.opacity);
                pixelBg.blend_up(imagePixel.background_color, self.opacity);
                buff.buffer[idx] = Pixel::new(Some(pixelFg), Some(pixelBg), None);
            }
        }
    }
    fn as_any(&self) -> &dyn Any {self}
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

// ComposedLayer
pub struct ComposedLayer {
    renderers: Vec<Box<dyn Element>>
}
/*
impl ComposedLayer {
    pub fn new() -> ComposedLayer {
        ComposedLayer { renderers: Vec::new() }
    }
    pub fn append(&mut self, element: Box<dyn Element>) -> usize {
        self.renderers.push(element);
        self.renderers.len()-1
    }
    pub fn pop(&mut self) -> Option<Box<dyn Element>> {self.renderers.pop()}
    pub fn get_mut(&mut self, i: usize) -> Option<Box<&mut dyn Element>> {
        self.renderers.get_mut(i)
    }
}
impl Element for ComposedLayer {
    fn process(&mut self, buff: &mut FrameBuffer) {
        let len = self.renderers.len();
        for i in 0..len {
            let mut renderer = self.renderers.get_mut(i).unwrap();
            renderer.process(buff);
        }
    }
    fn as_any(&self) -> &dyn Any {self}
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}
*/