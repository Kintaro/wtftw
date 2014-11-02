use window_manager::WindowManager;

pub struct RationalRect(f32, f32, f32, f32);

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
