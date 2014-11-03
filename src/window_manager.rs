use window_system::Rectangle;
use window_system::Window;

pub type Screen = Rectangle;

pub struct WindowManager {
    active_screen: uint
}

impl WindowManager {
    pub fn new() -> WindowManager {
        WindowManager {
            active_screen: 0
        }
    }

    pub fn get_active_screen(&self) -> uint {
        self.active_screen
    }

    pub fn manage(&mut self, window: Window) {
        
    }
}
