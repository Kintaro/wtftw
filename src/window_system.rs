pub enum WindowSystemEvent {
    WindowCreated(u64),
    WindowDestroyed(u64),
    Enter(u64),
    Leave(u64),
    UnknownEvent
}

pub trait WindowSystem {
    fn get_display_width(&self, screen: uint) -> u32;
    fn get_display_height(&self, screen: uint) -> u32;
    fn get_window_name(&self, window: u64) -> String;
    fn set_window_border_width(&mut self, window: u64, border_width: uint);
    fn set_window_border_color(&mut self, window: u64, border_color: uint);
    fn resize_window(&mut self, window: u64, width: u32, height: u32);
    fn move_window(&mut self, window: u64, x: u32, height: u32);
    fn show_window(&mut self, window: u64);
    fn event_pending(&self) -> bool;
    fn get_event(&mut self) -> WindowSystemEvent;
}


