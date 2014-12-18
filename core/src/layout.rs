use std::num::Float;
use std::mem;
use std::collections::EnumSet;
use std::collections::enum_set::CLike;
use core::Stack;
use std::num::Int;
use window_system::Window;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_manager::ScreenDetail;

#[deriving(Clone, Copy)]
pub enum LayoutMessage {
    Increase,
    Decrease,
    IncreaseMaster,
    DecreaseMaster,
    Next,
    Prev,
    HorizontalSplit,
    VerticalSplit
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
        .chain(split_vertically(num - 1, Rectangle(sx, sy + smallh, sw, sh - smallh)).iter())
        .map(|&x| x)
        .collect()
}

pub fn split_horizontally_by(ratio: f32, screen: ScreenDetail) -> (Rectangle, Rectangle) {
    let Rectangle(sx, sy, sw, sh) = screen;
    let leftw = (sw as f32 * ratio).floor() as u32;

    (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw, sy, sw - leftw, sh))
}

pub trait Layout {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)>;
    fn apply_message(&mut self, message: LayoutMessage) -> bool;
    fn description(&self) -> String;
    fn copy<'a>(&self) -> Box<Layout + 'a> { panic!("") }
}

#[deriving(Clone, Copy)]
pub struct TallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32
}

impl TallLayout {
    pub fn new<'a>() -> Box<Layout + 'a> {
        box TallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5
        } as Box<Layout + 'a>
    }
}

impl Layout for TallLayout {
    fn apply_layout(&self, _: &WindowSystem, screen: Rectangle,
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

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
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
        String::from_str("Tall")
    }

    fn copy<'a>(&self) -> Box<Layout + 'a> {
        box self.clone()
    }
}

#[deriving(Clone)]
pub struct ResizableTallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32,
    pub slaves: Vec<f32>
}

impl ResizableTallLayout {
    pub fn new<'a>() -> Box<Layout + 'a> {
        box ResizableTallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5,
            slaves: Vec::new()
        } as Box<Layout + 'a>
    }
}

impl Layout for ResizableTallLayout {
    fn apply_layout(&self, _: &WindowSystem, screen: Rectangle,
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

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
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
        String::from_str("ResizeTall")
    }

    fn copy<'a>(&self) -> Box<Layout + 'a> {
        box self.clone()
    }
}

/// A simple layout container that just
/// rotates the layout of its contained layout
/// by 90° clockwise
pub struct MirrorLayout<'a> {
    pub layout: Box<Layout + 'a>
}

impl<'a> MirrorLayout<'a> {
    /// Create a new MirrorLayout containing the given layout
    pub fn new(layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        box MirrorLayout { layout: layout }
    }
}

impl<'a> Layout for MirrorLayout<'a> {
    fn apply_layout(&self, w: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        // Rotate the screen, apply the layout, ...
        self.layout.apply_layout(w, mirror_rect(&screen), stack).iter()
            // and then rotate all resulting windows by 90° clockwise
            .map(|&(w, r)| (w, mirror_rect(&r))).collect()
    }

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
        self.layout.apply_message(message)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box MirrorLayout { layout: self.layout.copy() }
    }
}

#[repr(uint)]
#[deriving(Clone, Copy)]
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
        match v {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right
        }
    }
}

#[deriving(Clone, Copy)]
pub struct Strut(Direction, u64, u64, u64);

fn parse_strut_partial(x: Vec<u64>) -> Vec<Strut> {
    match x.as_slice() {
        [l, r, t, b, ly1, ly2, ry1, ry2, tx1, tx2, bx1, bx2] => {
            (vec!(Strut(Direction::Left, l, ly1, ly2),
                  Strut(Direction::Right, r, ry1, ry2),
                  Strut(Direction::Up, t, tx1, tx2),
                  Strut(Direction::Down, b, bx1, bx2))).into_iter()
                .filter(|&Strut(_, n, _, _)| n != 0)
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
                let s = vec!(Int::min_value(), Int::max_value());
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

/// A layout that avoids dock like windows (e.g. dzen, xmobar, ...)
/// to not overlap them.
pub struct AvoidStrutsLayout<'a> {
    directions: EnumSet<Direction>,
    layout: Box<Layout + 'a>
}

impl<'a> AvoidStrutsLayout<'a> {
    /// Create a new AvoidStrutsLayout, containing the given layout
    /// and avoiding struts in the given directions.
    pub fn new(d: Vec<Direction>, layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        box AvoidStrutsLayout {
            directions: d.iter().map(|&x| x).collect(),
            layout: layout.copy()
        }
    }
}

impl<'a> Layout for AvoidStrutsLayout<'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {

        let new_screen = stack.clone().map_or(screen, |_| {
            window_system.get_windows().into_iter()
                .filter(|&w| window_system.is_dock(w) && 
                        window_system.get_geometry(w).overlaps(&screen))
                .flat_map(|x| get_strut(window_system, x).into_iter())
                .filter(|&Strut(s, _, _, _)| self.directions.contains(&s))
                .fold(screen, |Rectangle(x, y, w, h), Strut(d, sw, _, _)| {
                    let s = sw as u32;
                    match d {
                        Direction::Up    => Rectangle(x, y + s, w, h - s),
                        Direction::Down  => Rectangle(x, y, w, h - s),
                        Direction::Left  => Rectangle(x + s, y, w - s, h),
                        Direction::Right => Rectangle(x, y, w - s, h)
                    }
                })
        });

        self.layout.apply_layout(window_system, new_screen, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
        self.layout.apply_message(message)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box AvoidStrutsLayout {
            directions: self.directions.clone(),
            layout: self.layout.copy()
        }
    }
}

pub struct GapLayout<'a> {
    gap: u32,
    layout: Box<Layout + 'a>
}

impl<'a> GapLayout<'a> {
    pub fn new(gap: u32, layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        box GapLayout {
            gap: gap,
            layout: layout.copy()
        }
    }
}

impl<'a> Layout for GapLayout<'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        let layout = self.layout.apply_layout(window_system, screen, stack);

        let g = self.gap / 2;
        layout.iter().map(|&(win, Rectangle(x, y, w, h))| (win, Rectangle(x + g, y + g, w - 2 * g, h - 2 * g))).collect()
    }

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
        self.layout.apply_message(message)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box GapLayout {
            gap: self.gap,
            layout: self.layout.copy()
        }
    }
}

#[deriving(Clone)]
pub enum SplitBox {
    Horizontal(Box<SplitBox>, Box<SplitBox>, Window, Window),
    Vertical(Box<SplitBox>, Box<SplitBox>, Window, Window),
    Single(Window),
    None
}

#[deriving(Clone)]
pub struct SplitLayout {
    pub root: SplitBox
}

impl SplitLayout {
    pub fn new<'a>() -> Box<Layout + 'a> {
        box SplitLayout { root: SplitBox::None }
    }
}

impl Layout for SplitLayout {
    fn apply_layout(&self, _: &WindowSystem, _: Rectangle,
                    _: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        Vec::new()
    }

    fn apply_message(&mut self, _: LayoutMessage) -> bool {
        true
    }

    fn description(&self) -> String {
        String::from_str("Split")
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box self.clone()
    }
}

#[deriving(Copy, Clone)]
pub struct FullLayout;

impl Layout for FullLayout {
    fn apply_layout(&self, _: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref st) => vec!((st.focus, screen)),
            None     => Vec::new()
        }
    }

    fn apply_message(&mut self, _: LayoutMessage) -> bool {
        true
    }

    fn description(&self) -> String {
        String::from_str("Split")
    }

    fn copy<'a>(&self) -> Box<Layout + 'a> {
        box self.clone()
    }
}

pub struct LayoutCollection<'a> {
    pub layouts: Vec<Box<Layout + 'a>>,
    pub current: uint
}

impl<'a> LayoutCollection<'a> {
    pub fn new(layouts: Vec<Box<Layout + 'a>>) -> Box<Layout + 'a> {
        box LayoutCollection {
            layouts: layouts,
            current: 0
        }
    }
}

impl<'a> Layout for LayoutCollection<'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        self.layouts[self.current].apply_layout(window_system, screen, stack)
    }

    fn apply_message(&mut self, message: LayoutMessage) -> bool {
        match message {
            LayoutMessage::Next => { self.current = (self.current + 1) % self.layouts.len(); true }
            LayoutMessage::Prev => { self.current = (self.current + (self.layouts.len() - 1)) % self.layouts.len(); true }
            _                   => self.layouts[self.current].apply_message(message)
        }
    }

    fn description(&self) -> String {
        self.layouts[self.current].description()
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box LayoutCollection {
            current: self.current,
            layouts: self.layouts.iter().map(|x| x.copy()).collect()
        }
    }
}
