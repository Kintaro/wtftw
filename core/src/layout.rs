extern crate collections;

use std::iter;
use std::num::Float;
use std::ops::Deref;
use self::collections::EnumSet;
use self::collections::enum_set::CLike;
use core::Stack;
use std::num::Int;
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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)>;
    fn apply_message<'b>(&mut self, _: LayoutMessage, _: &WindowSystem,
                         _: &Option<Stack<Window>>, _: &GeneralConfig<'b>) -> bool { true }
    fn description(&self) -> String;
    fn copy<'a>(&self) -> Box<Layout + 'a> { panic!("") }
    fn unhook<'b>(&self, _: &WindowSystem, _: &Option<Stack<Window>>, _: &GeneralConfig<'b>) { }
}

#[derive(Clone, Copy)]
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
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
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

    fn apply_message<'b>(&mut self, message: LayoutMessage, _: &WindowSystem,
                         _: &Option<Stack<Window>>, _: &GeneralConfig<'b>) -> bool {
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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                if s.len() == 1 {
                    self.layout.apply_layout(window_system, screen, config, &Some(s.clone()))
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
                    }).into_iter()).chain(self.layout.apply_layout(window_system, screen, config,
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

#[derive(Clone)]
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

    fn tile<U>(ratio: f32, mf: U, screen: ScreenDetail, num_master: u32, num_windows: u32) -> Vec<Rectangle> where
            U : Iterator<Item=f32> {
        if num_windows <= num_master || num_master == 0 {
            ResizableTallLayout::split_vertically(mf, num_windows, screen)
        } else {
            let v = mf.collect::<Vec<_>>();
            let (r1, r2) = ResizableTallLayout::split_horizontally_by(ratio, screen);
            let v1 = ResizableTallLayout::split_vertically(v.clone().into_iter(), num_master, r1);
            let v2 = ResizableTallLayout::split_vertically(v.clone().into_iter().skip(num_master as usize), num_windows - num_master, r2);
            v1.iter().chain(v2.iter()).map(|&x| x).collect()
        }
    }

    fn split_vertically<U>(r: U, num: u32, screen: ScreenDetail) -> Vec<Rectangle> where
            U : Iterator<Item=f32> {
        if r.size_hint().0 == 0 {
            return vec!(screen);
        }

        if num < 2 {
            return vec!(screen);
        }

        let Rectangle(sx, sy, sw, sh) = screen;
        let fxv = r.collect::<Vec<_>>();
        let f = fxv[0];
        let smallh = ((sh / num) as f32 * f) as u32;
        (vec!(Rectangle(sx, sy, sw, smallh))).iter()
            .chain(ResizableTallLayout::split_vertically(fxv.into_iter().skip(1), num - 1,
                                                         Rectangle(sx, sy + smallh, sw, sh - smallh)).iter())
            .map(|&x| x)
            .collect()
    }

    fn split_horizontally_by(ratio: f32, screen: ScreenDetail) -> (Rectangle, Rectangle) {
        let Rectangle(sx, sy, sw, sh) = screen;
        let leftw = (sw as f32 * ratio).floor() as u32;

        (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw, sy, sw - leftw, sh))
    }

    fn resize(&mut self, stack: &Option<Stack<Window>>, d: f32) {
        fn modify<U>(v: U, d: f32, n: usize) -> Vec<f32> where U : Iterator<Item=f32> {
            if v.size_hint().0 == 0 { return Vec::new(); }
            if n == 0 {
                let frac = v.collect::<Vec<_>>();
                (vec!(frac[0] + d)).into_iter().chain(frac.into_iter().skip(1)).collect()
            } else {
                let frac = v.collect::<Vec<_>>();
                (vec!(frac[0])).into_iter()
                    .chain(modify(frac.into_iter().skip(1), d, n - 1).into_iter())
                    .collect()
            }
        }

        if let &Some(ref s) = stack {
            let n = s.up.len();
            let total = s.len();
            let pos = if n as u32 == self.num_master - 1 || n == total - 1 { n - 1 } else { n };
            let mfrac = modify(self.slaves.clone().into_iter().chain(iter::repeat(1.0)).take(total), d, pos);
            self.slaves = mfrac.into_iter().take(total).collect();
        }
    }
}

impl Layout for ResizableTallLayout {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                let ws = s.integrate();
                s.integrate().iter()
                    .zip(ResizableTallLayout::tile(self.ratio,
                                                   self.slaves.clone().into_iter().chain(iter::repeat(1.0)).take(ws.len()),
                                                   screen, self.num_master, ws.len() as u32).iter())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, _: &WindowSystem,
                         stack: &Option<Stack<Window>>, _: &GeneralConfig<'b>) -> bool {
        let d = self.increment_ratio;
        match message {
            LayoutMessage::Increase => { self.ratio += self.increment_ratio; true }
            LayoutMessage::Decrease => { self.ratio -= self.increment_ratio; true }
            LayoutMessage::IncreaseMaster => { self.num_master += 1; true }
            LayoutMessage::DecreaseMaster => {
                if self.num_master > 1 { self.num_master -= 1 } true
            }
            LayoutMessage::IncreaseSlave => { self.resize(stack,  d);
            debug!("slaves are {:?}", self.slaves); true }
            LayoutMessage::DecreaseSlave => { self.resize(stack, -d); true }
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
    fn apply_layout(&mut self, w: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        // Rotate the screen, apply the layout, ...
        self.layout.apply_layout(w, mirror_rect(&screen), config, stack).iter()
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

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box MirrorLayout { layout: self.layout.copy() }
    }
}

#[repr(uint)]
#[derive(Clone, Copy)]
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

    pub fn to_axis(&self) -> Axis {
        match self {
            &Direction::Up    => Axis::Horizontal,
            &Direction::Down  => Axis::Horizontal,
            &Direction::Left  => Axis::Vertical,
            &Direction::Right => Axis::Vertical
        }
    }
}

impl CLike for Direction {
    fn to_uint(&self) -> usize {
        *self as usize
    }

    fn from_uint(v: usize) -> Direction {
        match v {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right
        }
    }
}

#[derive(Clone, Copy)]
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

    fn parse_strut(x: Vec<u64>) -> Vec<Strut> {
        match x.as_slice() {
            [a, b, c, d] => {
                let t = vec!(a, b, c, d);
                let s = vec!(Int::min_value(), Int::max_value());
                let r : Vec<u64> = t.iter().chain(s.iter().cycle()).take(12).map(|&x| x).collect();
                parse_strut_partial(r)
            }
            _ => Vec::new()
        }
    }

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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
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

        self.layout.apply_layout(window_system, new_screen, config, stack)
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        let layout = self.layout.apply_layout(window_system, screen, config, stack);

        let g = self.gap;
        layout.iter().map(|&(win, Rectangle(x, y, w, h))| (win, Rectangle(x + g, y + g, w - 2 * g, h - 2 * g))).collect()
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        match message {
            LayoutMessage::IncreaseGap => { self.gap += 1; true }
            LayoutMessage::DecreaseGap => { self.gap -= 1; true }
            _                          => self.layout.apply_message(message, window_system, stack, config)
        }
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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        if let &Some(ref s) = stack {
            for window in s.integrate().into_iter() {
                window_system.set_window_border_width(window, self.border);
            }
        }
        self.layout.apply_layout(window_system, screen, config, stack)
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
                         stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
        self.layout.apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
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
                let Rectangle(_, _, w, h) = window_system.get_geometry(window);
                window_system.resize_window(window, w + 2 * config.border_width, h + 2 * config.border_width);
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

#[derive(Copy, Clone)]
pub struct FullLayout;

impl Layout for FullLayout {
    fn apply_layout(&mut self, w: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
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
        String::from_str("Full")
    }

    fn copy<'a>(&self) -> Box<Layout + 'a> {
        box self.clone()
    }
}

pub struct LayoutCollection<'a> {
    pub layouts: Vec<Box<Layout + 'a>>,
    pub current: usize
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
    fn apply_layout(&mut self, window_system: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        self.layouts[self.current].apply_layout(window_system, screen, config, stack)
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

    fn copy<'b>(&self) -> Box<Layout + 'b> {
        box LayoutCollection {
            current: self.current,
            layouts: self.layouts.iter().map(|x| x.copy()).collect()
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical
}

impl Axis {
    pub fn opposite(&self) -> Axis {
        match self {
            &Axis::Horizontal => Axis::Vertical,
            &Axis::Vertical   => Axis::Horizontal
        }
    }
}

#[derive(Clone)]
pub enum Tree<T> {
    Leaf,
    Node(T, Box<Tree<T>>, Box<Tree<T>>)
}

impl<T> Tree<T> {
    pub fn number_of_leaves(&self) -> usize {
        match self {
            &Tree::Leaf => 1,
            &Tree::Node(_, ref l, ref r) => l.number_of_leaves() + r.number_of_leaves()
        }
    }
}

#[derive(Clone)]
pub struct Split {
    axis: Axis,
    ratio: f32
}

impl Split {
    pub fn new(axis: Axis, r: f32) -> Split {
        Split { axis: axis, ratio: r }
    }
    
    pub fn split(&self, Rectangle(x, y, w, h): Rectangle) -> (Rectangle, Rectangle) {
        match self.axis {
            Axis::Horizontal => {
                let hr = (h as f32 * self.ratio) as u32;
                (Rectangle(x, y, w, hr), Rectangle(x, y + hr, w, h - hr))
            },
            Axis::Vertical => {
                let wr = (w as f32 * self.ratio) as u32;
                (Rectangle(x, y, wr, h), Rectangle(x + wr, y, w - wr, h))
            }
        }
    }

    pub fn opposite(&self) -> Split {
        Split { axis: self.axis.opposite(), ratio: self.ratio }
    }

    pub fn increase_ratio(&self, r: f32) -> Split {
        Split { axis: self.axis.clone(), ratio: self.ratio + r }
    }
}

#[derive(Clone)]
pub enum Crumb<T> {
    LeftCrumb(T, Tree<T>),
    RightCrumb(T, Tree<T>)
}

impl<T: Clone> Crumb<T> {
    pub fn swap(&self) -> Crumb<T> {
        match self {
            &Crumb::LeftCrumb(ref s, ref t)  => Crumb::RightCrumb(s.clone(), t.clone()),
            &Crumb::RightCrumb(ref s, ref t) => Crumb::LeftCrumb(s.clone(), t.clone())
        }
    }

    pub fn parent(&self) -> T {
        match self {
            &Crumb::LeftCrumb(ref s, _)  => s.clone(),
            &Crumb::RightCrumb(ref s, _) => s.clone()
        }
    }

    pub fn modify_parent<F>(&self, f: F) -> Crumb<T> where F: Fn(&T) -> T {
        match self {
            &Crumb::LeftCrumb(ref s, ref t)  => Crumb::LeftCrumb(f(s), t.clone()),
            &Crumb::RightCrumb(ref s, ref t) => Crumb::RightCrumb(f(s), t.clone())
        }
    }
}

#[derive(Clone)]
pub struct Zipper {
    tree: Tree<Split>,
    crumbs: Vec<Crumb<Split>>
}

impl Zipper {
    fn left_append<S>(x: S, v: Vec<S>) -> Vec<S> {
        (vec!(x)).into_iter().chain(v.into_iter()).collect()
    }

    pub fn from_tree(tree: Tree<Split>) -> Zipper {
        Zipper {
            tree: tree.clone(),
            crumbs: Vec::new()
        }
    }

    pub fn go_left(&self) -> Option<Zipper> {
        match &self.tree {
            &Tree::Leaf => None,
            &Tree::Node(ref x, ref l, ref r) => Some(Zipper { 
                tree: l.deref().clone(), 
                crumbs: Zipper::left_append::<Crumb<Split>>(Crumb::LeftCrumb(x.clone(), r.deref().clone()), self.crumbs.clone()) 
            })
        }
    }
    
    pub fn go_right(&self) -> Option<Zipper> {
        match &self.tree {
            &Tree::Leaf => None,
            &Tree::Node(ref x, ref l, ref r) => Some(Zipper { 
                tree: r.deref().clone(), 
                crumbs: Zipper::left_append::<Crumb<Split>>(Crumb::RightCrumb(x.clone(), l.deref().clone()), self.crumbs.clone()) 
            })
        }
    }

    pub fn go_up(&self) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            None
        } else {
            let head = self.crumbs[0].clone();
            let rest = if self.crumbs.len() == 1 { Vec::new() } else { self.crumbs.clone().split_off(1) };

            match head {
                Crumb::LeftCrumb(x, r)  => Some(Zipper { tree: Tree::Node(x, box self.tree.clone(), box r), crumbs: rest }),
                Crumb::RightCrumb(x, l) => Some(Zipper { tree: Tree::Node(x, box l, box self.tree.clone()), crumbs: rest })
            }
        }
    }

    pub fn go_sibling(&self) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return None;
        }

        let head = self.crumbs[0].clone();

        match head {
            Crumb::LeftCrumb(_, _) => self.go_up().and_then(|x| x.go_right()),
            Crumb::RightCrumb(_, _) => self.go_up().and_then(|x| x.go_left())
        }
    }

    pub fn go_to_nth_leaf(&self, n: usize) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => Some(self.clone()),
            Tree::Node(_, ref l, _)  => {
                if l.number_of_leaves() > n {
                    self.go_left().and_then(|x| x.go_to_nth_leaf(n))
                } else {
                    self.go_right().and_then(|x| x.go_to_nth_leaf(n - l.number_of_leaves()))
                }
            }
        }
    }

    pub fn split_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Node(Split::new(Axis::Vertical, 0.5), box Tree::Leaf, box Tree::Leaf), crumbs: Vec::new() })
                } else {
                    let head = self.crumbs[0].clone();
                    Some(Zipper { 
                        tree: Tree::Node(Split::new(head.parent().axis.opposite(), 0.5), box Tree::Leaf, box Tree::Leaf), 
                        crumbs: self.crumbs.clone() 
                    })
                }
            }
            _ => None
        }
    }

    pub fn remove_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    None
                } else {
                    let head = self.crumbs[0].clone();
                    let rest = if self.crumbs.len() == 1 { Vec::new() } else {  self.crumbs.clone().split_off(1) };
                    match head {
                        Crumb::LeftCrumb(_, r) => Some(Zipper { tree: r.clone(), crumbs: rest }),
                        Crumb::RightCrumb(_, l) => Some(Zipper { tree: l.clone(), crumbs: rest })
                    }
                }
            },
            _ => None
        }
    }
    
    pub fn rotate_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Leaf, crumbs: Vec::new() })
                } else {
                    let mut c = self.crumbs.clone();
                    c[0] = c[0].modify_parent(|x| x.opposite());
                    Some(Zipper { 
                        tree: Tree::Leaf, 
                        crumbs: c 
                    })
                }
            }
            _ => None
        }
    }
    
    pub fn swap_current_leaf(&self) -> Option<Zipper> {
        match self.tree {
            Tree::Leaf => {
                if self.crumbs.is_empty() {
                    Some(Zipper { tree: Tree::Leaf, crumbs: Vec::new() })
                } else {
                    let mut c = self.crumbs.clone();
                    c[0] = c[0].swap();
                    Some(Zipper { 
                        tree: Tree::Leaf, 
                        crumbs: c 
                    })
                }
            }
            _ => None
        }
    }

    pub fn is_all_the_way(&self, dir: Direction) -> bool {
        if self.crumbs.is_empty() {
            return true;
        }

        let head = self.crumbs[0].clone();
        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Vertical   => false,
            (Direction::Left,  Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Vertical   => false,
            (Direction::Down,  Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Horizontal => false,
            (Direction::Up,    Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Horizontal => false,
            _ => self.go_up().map_or(false, |x| x.is_all_the_way(dir))
        }
    }

    pub fn expand_towards(&self, dir: Direction) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return Some(self.clone());
        }

        if self.is_all_the_way(dir) {
            return None;
        }

        let head = self.crumbs[0].clone();
        let rest = if self.crumbs.len() == 1 { Vec::new() } else { self.crumbs.clone().split_off(1) };

        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, ref r)) if s.axis == Axis::Vertical => Some(Zipper { 
                tree: self.tree.clone(), 
                crumbs: Zipper::left_append(Crumb::LeftCrumb(s.increase_ratio(0.05), r.clone()), rest)
            }),
            (Direction::Left, Crumb::RightCrumb(ref s, ref r)) if s.axis == Axis::Vertical => Some(Zipper { 
                tree: self.tree.clone(), 
                crumbs: Zipper::left_append(Crumb::RightCrumb(s.increase_ratio(-0.05), r.clone()), rest)
            }),
            (Direction::Down, Crumb::LeftCrumb(ref s, ref r)) if s.axis == Axis::Horizontal => Some(Zipper { 
                tree: self.tree.clone(), 
                crumbs: Zipper::left_append(Crumb::LeftCrumb(s.increase_ratio(0.05), r.clone()), rest)
            }),
            (Direction::Up, Crumb::RightCrumb(ref s, ref r)) if s.axis == Axis::Horizontal => Some(Zipper { 
                tree: self.tree.clone(), 
                crumbs: Zipper::left_append(Crumb::RightCrumb(s.increase_ratio(-0.05), r.clone()), rest)
            }),
            _ => self.go_up().and_then(|x| x.expand_towards(dir))
        }
    }

    pub fn shrink_from(&self, dir: Direction) -> Option<Zipper> {
        if self.crumbs.is_empty() {
            return Some(self.clone());
        }

        let head = self.crumbs[0].clone();

        match (dir, head) {
            (Direction::Right, Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Vertical   => self.go_sibling().and_then(|x| x.expand_towards(Direction::Left)),
            (Direction::Left,  Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Vertical   => self.go_sibling().and_then(|x| x.expand_towards(Direction::Right)),
            (Direction::Down,  Crumb::LeftCrumb(ref s, _))  if s.axis == Axis::Horizontal => self.go_sibling().and_then(|x| x.expand_towards(Direction::Up)),
            (Direction::Up,    Crumb::RightCrumb(ref s, _)) if s.axis == Axis::Horizontal => self.go_sibling().and_then(|x| x.expand_towards(Direction::Down)),
            _ => self.go_up().and_then(|x| x.shrink_from(dir))
        }
    }

    pub fn top(&self) -> Zipper {
        self.go_up().map_or(self.clone(), |x| x.top())
    }

    pub fn to_tree(&self) -> Tree<Split> {
        self.top().tree.clone()
    }
}

#[derive(Clone)]
pub struct BinarySpacePartition {
    tree: Option<Tree<Split>>
}

impl BinarySpacePartition {
    pub fn new<'a>() -> Box<Layout + 'a> {
        box BinarySpacePartition::empty()
    }

    pub fn empty() -> BinarySpacePartition {
        BinarySpacePartition { tree: None }
    }

    pub fn make(tree: Tree<Split>) -> BinarySpacePartition {
        BinarySpacePartition { tree: Some(tree) }
    }

    pub fn make_zipper(&self) -> Option<Zipper> {
        self.tree.clone().map(|x| Zipper::from_tree(x))
    }

    pub fn size(&self) -> usize {
        self.tree.clone().map_or(0, |x| x.number_of_leaves())
    }

    pub fn from_zipper(zipper: Option<Zipper>) -> BinarySpacePartition {
        BinarySpacePartition {
            tree: zipper.clone().map(|x| x.top().to_tree())
        }
    }

    pub fn rectangles(&self, rect: Rectangle) -> Vec<Rectangle> {
        self.tree.clone().map_or(Vec::new(), |t| {
            match t {
                Tree::Leaf => vec!(rect),
                Tree::Node(value, l, r) => {
                    let (left_box, right_box) = value.split(rect);
                    let left  = BinarySpacePartition::make(l.deref().clone()).rectangles(left_box);
                    let right = BinarySpacePartition::make(r.deref().clone()).rectangles(right_box);
                    left.into_iter().chain(right.into_iter()).collect()
                }
            }
        })
    }

    pub fn do_to_nth<F>(&self, n: usize, f: F) -> BinarySpacePartition where F: Fn(Zipper) -> Option<Zipper> {
        BinarySpacePartition::from_zipper(self.make_zipper().and_then(|x| x.go_to_nth_leaf(n)).and_then(f))
    }

    pub fn split_nth(&self, n: usize) -> BinarySpacePartition {
        if self.tree.is_none() {
            BinarySpacePartition::make(Tree::Leaf)
        } else {
            self.do_to_nth(n, |x| x.split_current_leaf())
        }
    }

    pub fn remove_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => BinarySpacePartition::empty(),
                    _           => self.do_to_nth(n, |x| x.remove_current_leaf())
                }
            }
        }
    }

    pub fn rotate_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.rotate_current_leaf())
                }
            }
        }
    }
    
    pub fn swap_nth(&self, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.swap_current_leaf())
                }
            }
        }
    }

    pub fn grow_nth_towards(&self, dir: Direction, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.expand_towards(dir))
                }
            }
        }
    }
    
    pub fn shrink_nth_from(&self, dir: Direction, n: usize) -> BinarySpacePartition {
        match self.tree {
            None => BinarySpacePartition::empty(),
            Some(ref tree) => {
                match tree {
                    &Tree::Leaf => self.clone(),
                    _           => self.do_to_nth(n, |x| x.shrink_from(dir))
                }
            }
        }

    }

    fn to_index<T: Clone + Eq>(s: Option<Stack<T>>) -> (Vec<T>, Option<usize>) {
        match s {
            None => (Vec::new(), None),
            Some(x) => (x.integrate(), Some(x.up.len()))
        }
    }

    fn stack_index<T: Clone + Eq>(s: &Stack<T>) -> usize {
        match BinarySpacePartition::to_index(Some(s.clone())) {
            (_, None) => 0,
            (_, Some(x)) => x
        }
    }
}

impl Layout for BinarySpacePartition {
    fn apply_layout(&mut self, _: &WindowSystem, screen: Rectangle, config: &GeneralConfig,
                    stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match *stack {
            Some(ref st) => {
                debug!("{:?}", st.integrate());
                let ws = st.integrate();

                fn layout(bsp: BinarySpacePartition, l: usize, n: usize) -> Option<BinarySpacePartition> {
                    if l == bsp.size() {
                        Some(bsp.clone())
                    } else if l > bsp.size() {
                        layout(bsp.split_nth(n), l, n)
                    } else {
                        layout(bsp.remove_nth(n), l, n)
                    }
                }

                let bsp = layout(self.clone(), ws.len(), BinarySpacePartition::stack_index(st));;

                let rs = match bsp {
                    None => self.rectangles(screen),
                    Some(ref b) => b.rectangles(screen)
                };
                if let Some(ref t) = bsp.clone() {
                    self.tree =  t.tree.clone();
                }

                ws.into_iter().zip(rs.into_iter()).collect()
            },
            None     => Vec::new()
        }
    }

    fn apply_message<'b>(&mut self, message: LayoutMessage, window_system: &WindowSystem,
        stack: &Option<Stack<Window>>, config: &GeneralConfig<'b>) -> bool {
            match message {
                LayoutMessage::TreeRotate => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.rotate_nth(index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }
                },
                LayoutMessage::TreeSwap => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.swap_nth(index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }
                },
                LayoutMessage::TreeExpandTowards(dir) => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.grow_nth_towards(dir, index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }

                },
                LayoutMessage::TreeShrinkFrom(dir) => {
                    if let &Some(ref s) = stack {
                        let index = BinarySpacePartition::stack_index(s);
                        let r = self.shrink_nth_from(dir, index);
                        self.tree = r.tree.clone();
                        true
                    } else {
                        false
                    }

                },
                _ => false
            }
        }

    fn description(&self) -> String {
        String::from_str("BSP")
    }

    fn copy<'a>(&self) -> Box<Layout + 'a> {
        box self.clone()
    }
}
