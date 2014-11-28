use std::num::Float;
use std::mem;
use std::collections::EnumSet;
use std::collections::enum_set::CLike;
use core::Stack;
use std::u64;
use std::num::Int;
use std::num::Bounded;
use window_system::Window;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_manager::ScreenDetail;

pub enum LayoutMessage {
    Increase,
    Decrease
}

pub struct RationalRect(f32, f32, f32, f32);

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
        .chain(split_vertically(num - 1, Rectangle(sx, sy + smallh, sw, sh - smallh)).iter())
        .map(|&x| x)
        .collect()
}

pub fn split_horizontally_by(ratio: f32, screen: ScreenDetail) -> (Rectangle, Rectangle) {
    let Rectangle(sx, sy, sw, sh) = screen;
    let leftw = (sw as f32 * ratio).floor() as u32;

    (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw, sy, sw - leftw, sh))
}

pub struct LayoutManager;

impl LayoutManager {
    pub fn get_layout<'a>(_: String) -> Box<Layout + 'static> {
        let tall = box TallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5
        };

        box AvoidStrutsLayout::new(vec!(Direction::Up), tall)
    }
}

pub trait Layout {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle, stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)>;
    fn apply_message(&mut self, message: LayoutMessage);
}

pub struct TallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32
}

impl Layout for TallLayout {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle, stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                debug!("Applying TallLayout to {} windows", s.integrate().len());
                let ws = s.integrate();
                s.integrate().iter()
                    .zip(tile(self.ratio, screen, self.num_master, ws.len() as u32).iter())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage) {
        match message {
            LayoutMessage::Increase => self.ratio += 0.05,
            LayoutMessage::Decrease => self.ratio -= 0.05
        }
    }
}

#[repr(uint)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl CLike for Direction {
    fn to_uint(&self) -> uint {
        *self as uint
    }

    fn from_uint(v: uint) -> Direction {
        unsafe { mem::transmute(v) }
    }
}

pub struct Strut(Direction, u64, u64, u64);

fn parse_strut_partial(x: Vec<u64>) -> Vec<Strut> {
    match x.as_slice() {
        [l, r, t, b, ly1, ly2, ry1, ry2, tx1, tx2, bx1, bx2] => {
            (vec!(Strut(Direction::Left, l, ly1, ly2),
                  Strut(Direction::Right, r, ry1, ry2),
                  Strut(Direction::Up, t, tx1, tx2),
                  Strut(Direction::Down, b, bx1, bx2))).iter()
                .filter(|&&Strut(_, n, _, _)| n != 0)
                .map(|x| *x.clone())
                .collect()
        },
        _ => Vec::new()
    }
}

pub fn get_strut(window_system: &WindowSystem, window: Window) -> Vec<Strut> {
    let partial_strut = window_system.get_partial_strut(window);

    let parse_strut = |x: Vec<u64>| {
        match x.as_slice() {
            [a, b, c, d] => {
                let t = vec!(a, b, c, d);
                let s = vec!(Bounded::min_value(), Bounded::max_value());
                let r : Vec<u64> = t.iter().chain(s.iter().cycle()).take(12).map(|&x| x).collect();
                parse_strut_partial(r)
            }
            _ => Vec::new()
        }
    };;

    match partial_strut {
        Some(ps) => parse_strut_partial(ps),
        None     => {
            let strut = window_system.get_strut(window);
            match strut {
                Some(s) => parse_strut(s),
                None    => Vec::new()
            }
        }
    }
}

pub struct AvoidStrutsLayout<'a> {
    directions: EnumSet<Direction>,
    layout: Box<Layout + 'a>
}

impl<'a> AvoidStrutsLayout<'a> {
    pub fn new<'b>(d: Vec<Direction>, layout: Box<Layout + 'b>) -> AvoidStrutsLayout<'b> {
        AvoidStrutsLayout {
            directions: d.iter().map(|&x| x).collect(),
            layout: layout
        }
    }
}

impl<'a> Layout for AvoidStrutsLayout<'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle, 
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        let windows = window_system.get_windows();
        let struts : Vec<Strut> = windows.iter()
            .flat_map(|&x| get_strut(window_system, x).into_iter())
            .filter(|&Strut(s, _, _, _)| self.directions.contains(&s))
            .collect();

        let new_screen = struts.iter().fold(screen, |Rectangle(x, y, w, h), &Strut(d, sw, _, _)| {
            let s = sw as u32;
            match d {
                Direction::Up    => Rectangle(x, y + s, w, h - s),
                Direction::Down  => Rectangle(x, y, w, h - s),
                Direction::Left  => Rectangle(x + s, y, w - s, h),
                Direction::Right => Rectangle(x, y, w - s, h)
            }
        });
        self.layout.apply_layout(window_system, new_screen, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage) {
        self.layout.apply_message(message)
    }
}
