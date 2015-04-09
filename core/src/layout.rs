extern crate num;

use std::borrow::ToOwned;
use core::stack::Stack;
use window_system::Window;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_manager::ScreenDetail;
use config::GeneralConfig;

#[derive(Clone, Copy)]
pub enum LayoutMessage {
    Increase,
    Decrease,
    IncreaseMaster,
    DecreaseMaster,
    IncreaseSlave,
    DecreaseSlave,
    IncreaseGap,
    DecreaseGap,
    Next,
    Prev,
    HorizontalSplit,
    VerticalSplit,
    Hide,
    TreeRotate,
    TreeSwap,
    TreeExpandTowards(Direction),
    TreeShrinkFrom(Direction)
}

pub fn mirror_rect(&Rectangle(x, y, w, h) : &Rectangle) -> Rectangle {
    Rectangle(y, x, h, w)
}

pub fn tile(ratio: f32, screen: ScreenDetail, num_master: u32, num_windows: u32) -> Vec<Rectangle> {
    if num_windows <= num_master || num_master == 0 {
        split_vertically(num_windows, screen)
    } else {
        let (r1, r2) = split_horizontally_by(ratio, screen);
        let v1 = split_vertically(num_master, r1);
        let v2 = split_vertically(num_windows - num_master, r2);
        v1.iter().chain(v2.iter()).map(|&x| x).collect()
    }
}

pub fn split_vertically(num: u32, screen: ScreenDetail) -> Vec<Rectangle> {
    if num < 2 {
        return vec!(screen);
    }

    let Rectangle(sx, sy, sw, sh) = screen;
    let smallh = sh / num;
    (vec!(Rectangle(sx, sy, sw, smallh))).iter()
        .chain(split_vertically(num - 1, Rectangle(sx, sy + smallh as i32, sw, sh - smallh)).iter())
        .map(|&x| x)
        .collect()
}

pub fn split_horizontally_by(ratio: f32, screen: ScreenDetail) -> (Rectangle, Rectangle) {
    let Rectangle(sx, sy, sw, sh) = screen;
    let leftw = (sw as f32 * ratio).floor() as u32;

    (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw as i32, sy, sw - leftw, sh))
}

pub trait Layout {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)>;
    fn apply_message(&mut self, _: LayoutMessage, _: &WindowSystem,
                         _: &Option<Stack<Window>>, _: &GeneralConfig) -> bool { true }
    fn description(&self) -> String;
    fn copy(&self) -> Box<Layout> { panic!("") }
    fn unhook(&self, _: &WindowSystem, _: &Option<Stack<Window>>, _: &GeneralConfig) { }
}

#[derive(Clone, Copy)]
pub struct TallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32
}

impl TallLayout {
    pub fn new() -> Box<Layout> {
        Box::new(TallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5
        })
    }
}

impl Layout for TallLayout {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, _: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                let ws = s.integrate();
                s.integrate().iter()
                    .zip(tile(self.ratio, screen, self.num_master, ws.len() as u32).iter())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage, _: &WindowSystem,
                         _: &Option<Stack<Window>>, _: &GeneralConfig) -> bool {
        match message {
            LayoutMessage::Increase => { self.ratio += 0.05; true }
            LayoutMessage::Decrease => { self.ratio -= 0.05; true }
            LayoutMessage::IncreaseMaster => { self.num_master += 1; true }
            LayoutMessage::DecreaseMaster => {
                if self.num_master > 1 { self.num_master -= 1 } true
            }
            _                       => false
        }
    }

    fn description(&self) -> String {
        "Tall".to_owned()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Ord, Eq, PartialOrd, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            &Direction::Up    => Direction::Down,
            &Direction::Down  => Direction::Up,
            &Direction::Left  => Direction::Right,
            &Direction::Right => Direction::Left
        }
    }
}

pub struct GapLayout {
    gap: u32,
    layout: Box<Layout>
}

impl GapLayout {
    pub fn new(gap: u32, layout: Box<Layout>) -> Box<Layout> {
        Box::new(GapLayout {
            gap: gap,
            layout: layout.copy()
        })
    }
}

impl Layout for GapLayout {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        let layout = self.layout.apply_layout(window_system, screen, config, stack);

        let g = self.gap;
        layout.iter().map(|&(win, Rectangle(x, y, w, h))| (win, Rectangle(x + g as i32, y + g as i32, w - 2 * g, h - 2 * g))).collect()
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        match message {
            LayoutMessage::IncreaseGap => { self.gap += 1; true }
            LayoutMessage::DecreaseGap => { if self.gap > 0 { self.gap -= 1; } true }
            _                          => self.layout.apply_message(message, window_system, stack, config)
        }
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(GapLayout {
            gap: self.gap,
            layout: self.layout.copy()
        })
    }
}

pub struct WithBordersLayout {
    border: u32,
    layout: Box<Layout>
}

impl WithBordersLayout {
    pub fn new(border: u32, layout: Box<Layout>) -> Box<Layout> {
        Box::new(WithBordersLayout {
            border: border,
            layout: layout.copy()
        })
    }
}

impl Layout for WithBordersLayout {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, self.border);
            }
        }
        self.layout.apply_layout(window_system, screen, config, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(WithBordersLayout {
            border: self.border,
            layout: self.layout.copy()
        })
    }

    fn unhook(&self, window_system: &WindowSystem, stack: &Option<Stack<Window>>, config: &GeneralConfig) {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, config.border_width);
                let Rectangle(_, _, w, h) = window_system.get_geometry(window);
                window_system.resize_window(window, w + 2 * config.border_width, h + 2 * config.border_width);
            }
        }
    }
}

pub struct NoBordersLayout;

impl NoBordersLayout {
    pub fn new(layout: Box<Layout>) -> Box<Layout> {
        WithBordersLayout::new(0, layout)
    }
}

#[derive(Copy, Clone)]
pub struct FullLayout;

impl Layout for FullLayout {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref st) => {
                let bw = 2 * config.border_width;
                let Rectangle(x, y, sw, sh) = screen;
                vec!((st.focus, Rectangle(x, y, sw + bw, sh + bw)))
            }
            None     => Vec::new()
        }
    }

    fn description(&self) -> String {
        "Full".to_owned()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}

pub struct LayoutCollection {
    pub layouts: Vec<Box<Layout>>,
    pub current: usize
}

impl LayoutCollection {
    pub fn new(layouts: Vec<Box<Layout>>) -> Box<Layout> {
        Box::new(LayoutCollection {
            layouts: layouts,
            current: 0
        })
    }
}

impl Layout for LayoutCollection {
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        self.layouts[self.current].apply_layout(window_system, screen, config, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig) -> bool {
        match message {
            LayoutMessage::Next => {
                self.layouts[self.current].unhook(window_system, stack, config);
                self.current = (self.current + 1) % self.layouts.len();
                true
            }
            LayoutMessage::Prev => {
                self.layouts[self.current].unhook(window_system, stack, config);
                self.current = (self.current + (self.layouts.len() - 1)) % self.layouts.len();
                true
            }
            _                   => self.layouts[self.current].apply_message(message, window_system, stack, config)
        }
    }

    fn description(&self) -> String {
        self.layouts[self.current].description()
    }

    fn copy(&self) -> Box<Layout> {
        Box::new(LayoutCollection {
            current: self.current,
            layouts: self.layouts.iter().map(|x| x.copy()).collect()
        })
    }
}
