
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
