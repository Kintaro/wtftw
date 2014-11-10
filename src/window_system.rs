#![allow(non_upper_case_globals)]
pub type Window = u64;

#[deriving(Show, Clone)]
pub struct Rectangle(pub u32, pub u32, pub u32, pub u32);

impl Rectangle {
    pub fn is_inside(&self, x: u32, y: u32) -> bool {
        let &Rectangle(rx, ry, rw, rh) = self;

        x >= rx && x <= rx + rw && y >= ry && y <= ry + rh 
    }
}

#[deriving(Clone)]
pub struct WindowChanges {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub border_width: u32,
    pub sibling: Window,
    pub stack_mode: u32,
}

/// Represents a keyboard input
/// with an abstracted modifier mask
/// and the key represented as a string
#[deriving(Show, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyCommand {
    pub mask: KeyModifiers,
    pub key: String
}

bitflags! {
    #[deriving(Show)]
    flags KeyModifiers : u32 {
        const NoneMask    = (0 << 0),
        const ShiftMask   = (1 << 0),
        const LockMask    = (1 << 1),
        const ControlMask = (1 << 2),
        const Mod1Mask    = (1 << 3),
        const Mod2Mask    = (1 << 4),
        const Mod3Mask    = (1 << 5),
        const Mod4Mask    = (1 << 6),
        const Mod5Mask    = (1 << 7),
    }
}

impl KeyModifiers {
    pub fn get_mask(&self) -> u32 {
        self.bits()
    }
}

#[deriving(Clone)]
pub enum WindowSystemEvent {
    ConfigurationNotification(Window),
    ConfigurationRequest(Window, WindowChanges, u64),
    /// A window has been created and needs to be managed.
    WindowCreated(Window),
    /// A window has been destroyed and needs to be unmanaged.
    WindowDestroyed(Window),
    WindowUnmapped(Window, bool),
    /// The pointer has entered a window's area. Mostly used
    /// for mousefollow focus.
    Enter(Window),
    /// The pointer has left a window's area. Mostly used
    /// for mousefollow focus.
    Leave(Window),
    ButtonPressed(Window, u32, u32, u32, u32),
    KeyPressed(Window, KeyCommand),
    ClientMessageEvent(Window),
    /// The underlying event by xlib or wayland is unknown
    /// and can be ignored.
    UnknownEvent
}

pub trait WindowSystem {
    fn get_string_from_keycode(&self, key: u32) -> String;
    fn get_keycode_from_string(&self, key: &String) -> u32; 
    fn get_root(&self) -> Window;
    /// Retrieve geometry infos over all screens
    fn get_screen_infos(&self) -> Vec<Rectangle>;
    /// Get the number of physical displays
    fn get_number_of_screens(&self) -> uint;
    /// Get the width of the given physical screen 
    fn get_display_width(&self, screen: uint) -> u32;
    /// Get the height of the given physical screen
    fn get_display_height(&self, screen: uint) -> u32;
    /// Get the given window's name
    fn get_window_name(&self, window: Window) -> String;
    /// Get a list of all windows
    fn get_windows(&self) -> Vec<Window>;
    /// Set the given window's border width
    fn set_window_border_width(&self, window: Window, border_width: u32);
    /// Set the given window's border color
    fn set_window_border_color(&self, window: Window, border_color: u32);
    /// Resize the window to the given dimensions
    fn resize_window(&self, window: Window, width: u32, height: u32);
    /// Move the window's top left corner to the given coordinates
    fn move_window(&self, window: Window, x: u32, height: u32);
    /// Map the window to the screen and show it
    fn show_window(&self, window: Window);
    fn hide_window(&self, window: Window);
    fn focus_window(&self, window: Window);
    fn get_focused_window(&self) -> Window;
    fn configure_window(&self, window: Window, window_changes: WindowChanges, mask: u64);
    /// Check if there are events pending
    fn event_pending(&self) -> bool;
    /// Get the next event from the queue
    fn get_event(&self) -> WindowSystemEvent;
    fn flush(&self);
    fn grab_keys(&self, keys: Vec<KeyCommand>);
}


