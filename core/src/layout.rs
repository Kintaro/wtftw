use std::num::Float;
use std::collections::EnumSet;
use std::collections::enum_set::CLike;
use core::Stack;
use std::num::Int;
use window_system::Window;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_manager::ScreenDetail;
use config::GeneralConfig;

#[deriving(Clone, Copy)]
pub enum LayoutMessage {
    Increase,
    Decrease,
    IncreaseMaster,
    DecreaseMaster,
    Next,
    Prev,
    HorizontalSplit,
    VerticalSplit,
    Hide
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
    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool;
    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
    }
    fn description(&self) -> String;
    fn copy<'a>(&self) -> Box<Layout + 'a> { panic!("") }
    fn unhook<'b>(&self, window_system: &WindowSystem, &Option<Stack<Window>>, config: &GeneralConfig<'b>) { }
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
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

pub struct CenterLayout<'a> {
    pub layout: Box<Layout + 'a>
}

impl<'a> CenterLayout<'a> {
    pub fn new(layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        box CenterLayout {
            layout: layout.copy()
        } as Box<Layout + 'a>
    }
}

impl<'a> Layout for CenterLayout <'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                if s.len() == 1 {
                    self.layout.apply_layout(window_system, screen, &Some(s.clone()))
                } else {
                    let new_stack = if s.up.len() > 0 {
                        Stack::<Window>::new(s.up[0], s.up.as_slice().tail().to_vec(), s.down.clone())
                    } else {
                        Stack::<Window>::new(s.down[0], Vec::new(), s.down.as_slice().tail().to_vec())
                    };
                    (vec!({
                        let x = screen.0 + ((screen.2 as f32 * 0.2) as u32 / 2);
                        let y = screen.1 + ((screen.3 as f32 * 0.2) as u32 / 2);
                        let w = (screen.2 as f32 * 0.8) as u32;
                        let h = (screen.3 as f32 * 0.8) as u32;
                        (s.focus, Rectangle(x, y, w, h))
                    }).into_iter()).chain(self.layout.apply_layout(window_system, screen,
                                                                   &Some(new_stack)).into_iter()).collect()
                }
            },
            _ => Vec::new()
        }
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        String::from_str("Center")
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        CenterLayout::new(self.layout.copy())
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        self.layout.post_apply_layout(window_system, screen, stack, config);
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        self.layout.post_apply_layout(window_system, screen, stack, config);
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        self.layout.post_apply_layout(window_system, screen, stack, config);
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box GapLayout {
            gap: self.gap,
            layout: self.layout.copy()
        }
    }
}

pub struct WithBordersLayout<'a> {
    border: u32,
    layout: Box<Layout + 'a>
}

impl<'a> WithBordersLayout<'a> {
    pub fn new(border: u32, layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        box WithBordersLayout {
            border: border,
            layout: layout.copy()
        }
    }
}

impl<'a> Layout for WithBordersLayout<'a> {
    fn apply_layout(&self, window_system: &WindowSystem, screen: Rectangle,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        self.layout.apply_layout(window_system, screen, stack)
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                //let Rectangle(x, y, w, h) = window_system.get_geometry(window);
                //let gap = 2 * (config.border_width - self.border);
                window_system.set_window_border_width(window, self.border);
                //window_system.resize_window(window, w + gap, h + gap);
            }
        }
        self.layout.post_apply_layout(window_system, screen, stack, config);
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box WithBordersLayout {
            border: self.border,
            layout: self.layout.copy()
        }
    }

    fn unhook<'b>(&self, window_system: &WindowSystem, stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, config.border_width);
            }
        }
    }
}

pub struct NoBordersLayout<'a>;

impl<'a> NoBordersLayout<'a> {
    pub fn new(layout: Box<Layout + 'a>) -> Box<Layout + 'a> {
        WithBordersLayout::new(0, layout)
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {

        true
    }

    fn description(&self) -> String {
        String::from_str("Full")
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
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

    fn post_apply_layout<'b>(&self, window_system: &WindowSystem, screen: Rectangle,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) {
        self.layouts[self.current].post_apply_layout(window_system, screen, stack, config);
    }

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box LayoutCollection {
            current: self.current,
            layouts: self.layouts.iter().map(|x| x.copy()).collect()
        }
    }
}