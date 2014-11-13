use core::Stack;
use window_system::Window;
use window_system::Rectangle;
use window_manager::ScreenDetail;

pub enum LayoutMessage {
    Increase,
    Decrease
}

pub struct RationalRect(f32, f32, f32, f32);

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
    pub fn get_layout(_: String) -> Box<Layout + 'static> {
        box TallLayout {
            num_master: 1,
            increment_ratio: 0.03,
            ratio: 0.5
        }
    }
}

pub trait Layout {
    fn apply_layout(&self, screen: Rectangle, stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)>;
    fn apply_message(&mut self, message: LayoutMessage);
}

pub struct TallLayout {
    pub num_master: u32,
    pub increment_ratio: f32,
    pub ratio: f32
}

impl Layout for TallLayout {
    fn apply_layout(&self, screen: Rectangle, stack: &Option<Stack<Window>>) -> Vec<(Window, Rectangle)> {
        match stack {
            &Some(ref s) => {
                debug!("Applying TallLayout to {} windows", s.integrate().len());
                let ws = s.integrate();
                ws.iter()
                    .zip(tile(self.ratio, screen, self.num_master, ws.len() as u32).iter())
                    .map(|(&x, &y)| (x, y))
                    .collect()
            },
            _ => Vec::new()
        }
    }

    fn apply_message(&mut self, message: LayoutMessage) {
        match message {
            Increase => self.ratio += 0.05,
            Decrease => self.ratio -= 0.05
        }
    }
}
