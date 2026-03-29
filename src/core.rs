use crossterm::{
    execute, queue,
    terminal::{size, SetSize, Clear, ClearType},
    style::{Print, SetBackgroundColor, SetForegroundColor, ResetColor, Color as TermColor},
    event::{self, Event, KeyCode, KeyEvent},
    cursor::{MoveTo, Hide}
};
use std::{array, collections::HashMap, hash::Hash, ops::DerefMut, thread::sleep, time::{Duration, Instant}};
use std::io::{stdout, Write};
use crate::{base::{
    Color, Element, FrameBuffer, Pixel, Position, RelPosition, RelSize, Size
}, elements::Label};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EventType {
    // app
    PreRender,
    PostRender,
    OnSizeChange,

    // keys
    Char { char: char },
    Enter,
    Backspace
}

pub struct EventContext {
    pub inner: HashMap<String, Box<dyn std::any::Any>>
}

impl EventContext {
    fn new() -> EventContext {
        EventContext { inner: HashMap::new() }
    }
    pub fn insert<T: 'static>(&mut self, key: &str, value: T) {
        self.inner.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.inner.get(key)?.downcast_ref::<T>()
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.inner.get_mut(key)?.downcast_mut::<T>()
    }
}

pub struct AppEvent {
    pub evtype: EventType,
    pub exec: Box<dyn FnMut(&mut HashMap<u32, Box<dyn Element>>, &mut FrameBuffer, &mut EventContext)>
}

pub struct PaletteCache {
    pub fg: HashMap<Color, String>,
    pub bg: HashMap<Color, String>
}

pub struct App {
    // args
    pub fps: u32,
    pub color_mask_max: Color,
    pub color_mask_min: Color,
    pub default_color: Color,
    pub default_color_bg: Color,
    pub input_time: f32,

    // vars
    pub renderers: HashMap<u32, Box<dyn Element>>,
    pub renderers_count: u32,
    pub running: bool,
    pub buffer: FrameBuffer,
    pub events: Vec<AppEvent>,
    pub event_context: EventContext,
    pub colorcache: PaletteCache
}

impl App {
    pub fn new(
        fps: u32,
        color_mask_max: Option<Color>,
        color_mask_min: Option<Color>,
        default_color: Option<Color>,
        default_color_bg: Option<Color>,
        input_time: Option<f32>
    ) -> App {
        let (w, h) = size().unwrap();
        App {
            fps,
            color_mask_max: color_mask_max.unwrap_or_else(Color::white),
            color_mask_min: color_mask_min.unwrap_or_else(Color::black),
            default_color: default_color.unwrap_or_else(Color::white),
            default_color_bg: default_color_bg.unwrap_or_else(Color::black),
            input_time: input_time.unwrap_or_else(|| 0.0001),
            renderers: HashMap::new(),
            renderers_count: 0 as u32,
            running: false,
            buffer: FrameBuffer::new(Size { width: w, height: h }, Some(Pixel::new(default_color, default_color_bg, None))),
            events: Vec::new(),
            event_context: EventContext::new(),
            colorcache: PaletteCache { fg: HashMap::new(), bg: HashMap::new() }
        }
    }

    pub fn init(fps: u32) -> App {
        App::new(fps, None, None, None, None, None)
    }

    pub fn register_event(&mut self, ev: AppEvent) {
        self.events.push(ev);
    }

    pub fn register_renderer(&mut self, renderer: Box<dyn Element>) -> u32 {
        self.renderers.insert(self.renderers_count, renderer);
        self.renderers_count += 1;
        self.renderers_count-1
    }

    pub fn get_renderer(&self, id: &u32) -> Option<&Box<dyn Element>> {
        self.renderers.get(id)
    }

    pub fn emit(&mut self, event: EventType) {
        let len = self.events.len();
        for i in 0..len {
            let mut ev = self.events.get_mut(i);
            match ev {
                Some(x) => {
                    if x.evtype == event {
                        (x.exec)(&mut self.renderers, &mut self.buffer, &mut self.event_context);
                    }
                },
                None => {}
            }
        }
    }

    pub fn get_events(&mut self) {
        if event::poll(Duration::from_secs_f32(self.input_time)).unwrap() {
            let ev = event::read().unwrap();
            match ev {
                Event::Key(KeyEvent {code, ..}) => match code {
                    KeyCode::Char(x) => {
                        self.emit(EventType::Char { char: x });
                    }
                    KeyCode::Enter => {
                        self.emit(EventType::Enter);
                    }
                    KeyCode::Backspace => {
                        self.emit(EventType::Backspace);
                    }
                    _ => {}
                }
                _ => {}
            }
        }
    }

    pub fn tick(&mut self) {
        let (screen_width, screen_height) = size().unwrap();
        if screen_width != self.buffer.size.width || screen_height != self.buffer.size.height {
            let size = Size { width: screen_width, height: screen_height };
            self.buffer.resize(size);
            self.emit(EventType::OnSizeChange);
        }

        self.get_events();
        let len = self.renderers.len();
        for i in 0..len {
            let renderer = self.renderers.get_mut(&(i as u32));
            match renderer {
                Some(renderer) => {renderer.process(&mut self.buffer);}
                None => {}
            };
        };
    }


    pub fn get_fg_str(&mut self, color: Color) -> &mut String {
        self.colorcache.fg.entry(color).or_insert_with(|| color.to_fg_string().to_string())
    }

    pub fn get_bg_str(&mut self, color: Color) -> &mut String {
        self.colorcache.bg.entry(color).or_insert_with(|| color.to_bg_string().to_string())
    }

    pub fn render(&mut self) {
        let mut stdout = stdout();
        
        execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
        let mut lastColor = self.default_color;
        let mut lastColorBg = self.default_color_bg;
        let firstPixel = match self.buffer.buffer.get(0) {Some(x) => x, None => {&Pixel { foreground_color: self.default_color, background_color: self.default_color_bg, char: ' ' }}};
        let firstPixelFg = firstPixel.foreground_color.maskmax(self.color_mask_max).maskmin(self.color_mask_min);
        let firstPixelBg = firstPixel.background_color.maskmax(self.color_mask_max).maskmin(self.color_mask_min);
        let mut frame = String::new();
        frame.push_str(self.get_bg_str(firstPixelBg));
        frame.push_str(self.get_fg_str(firstPixelFg));
        for y in 0..self.buffer.size.height as u16 {
            for x in 0..self.buffer.size.width as u16 {
                let idx = (y as usize) * (self.buffer.size.width as usize) + (x as usize);
                let pixel = self.buffer.buffer[idx];

                let fg = pixel.foreground_color.maskmax(self.color_mask_max).maskmin(self.color_mask_min);
                let bg = pixel.background_color.maskmax(self.color_mask_max).maskmin(self.color_mask_min);

                if lastColor != fg {
                    frame.push_str(self.get_fg_str(fg));
                    lastColor = fg;
                }
                if lastColorBg != bg {
                    frame.push_str(self.get_bg_str(bg));
                    lastColorBg = bg;
                }
                frame.push(pixel.char);
            }
        }
        queue!(
            stdout,
            Print(frame)
        ).unwrap();
        stdout.flush().unwrap();
    }

    fn apploop(&mut self) {
        let mut t1; let mut t2; let mut elapsed;
        let frametime = 1.0 / self.fps as f32;
        while self.running {
            t1 = Instant::now();

            self.emit(EventType::PreRender);
            self.tick();
            self.render();
            self.emit(EventType::PostRender);

            self.buffer.clear();

            t2 = Instant::now();
            elapsed = t2.duration_since(t1).as_secs_f32();
            if elapsed < frametime {
                sleep(Duration::from_secs_f32(frametime-elapsed));
            }
        }
    }

    pub fn run(&mut self) {
        execute!(stdout(), Hide);
        self.running = true;
        self.apploop();
    }
}