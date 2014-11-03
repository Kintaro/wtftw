use window_system::Rectangle;
use window_manager::ScreenDetail;
use window_manager::WindowManager;

pub struct RationalRect(f32, f32, f32, f32);

pub enum Resize {
    Shrink,
    Expand
}

struct IncreaseMasterClients(uint);

pub fn tile(ratio: f32, screen: ScreenDetail, num_master: uint, num_windows: uint) -> Vec<Rectangle> {
    if num_windows <= num_master || num_master == 0 {
        split_vertically(num_windows, screen)
    } else {
        let (r1, r2) = split_horizontally_by(ratio, screen);
        let v1 = split_vertically(num_master, r1);
        let v2 = split_vertically(num_windows - num_master, r2);
        v1.iter().chain(v2.iter()).map(|&x| x).collect()
    }
}

pub fn split_vertically(num: uint, screen: ScreenDetail) -> Vec<Rectangle> {
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
    let leftw = (sw as f32 * ratio).floor() as uint;

    (Rectangle(sx, sy, leftw, sh), Rectangle(sx + leftw, sy, sw - leftw, sh))
}

pub trait Layout {
    fn apply_layout(&self, window_manager: &WindowManager) -> Vec<RationalRect>; 
}

pub struct TallLayout {
    num_master: uint,
    increment_ratio: f32,
    ratio: f32
}

impl Layout for TallLayout {
    fn apply_layout(&self, window_manager: &WindowManager) -> Vec<RationalRect> {
        panic!("not yet implemented");
    }
}
