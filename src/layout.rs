use core::RationalRect;
use window_manager::WindowManager;

pub enum Resize {
    Shrink,
    Expand
}

struct IncreaseMasterClients(uint);

pub trait Layout {
    fn apply_layout(&self, window_manager: &WindowManager) -> Vec<RationalRect> {
        panic!("not yet implemented");
    }
}
