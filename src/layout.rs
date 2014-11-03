use window_manager::WindowManager;

pub struct RationalRect(f32, f32, f32, f32);

pub enum Resize {
    Shrink,
    Expand
}

struct IncreaseMasterClients(uint);

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
