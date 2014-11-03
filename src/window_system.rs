pub type Window = u64;

#[deriving(Show)]
pub struct Rectangle(pub uint, pub uint, pub uint, pub uint);

pub enum WindowSystemEvent {
    WindowCreated(Window),
    WindowDestroyed(Window),
    Enter(Window),
    Leave(Window),
    UnknownEvent
}

pub trait WindowSystem {
    fn get_screen_infos(&self) -> Vec<Rectangle>;
    fn get_number_of_screens(&self) -> uint;
    fn get_display_width(&self, screen: uint) -> u32;
    fn get_display_height(&self, screen: uint) -> u32;
    fn get_window_name(&self, window: Window) -> String;
    fn get_windows(&self) -> Vec<Window>;
    fn set_window_border_width(&mut self, window: Window, border_width: uint);
    fn set_window_border_color(&mut self, window: Window, border_color: uint);
    fn resize_window(&mut self, window: Window, width: u32, height: u32);
    fn move_window(&mut self, window: Window, x: u32, height: u32);
    fn show_window(&mut self, window: Window);
    fn event_pending(&self) -> bool;
    fn get_event(&mut self) -> WindowSystemEvent;
}


