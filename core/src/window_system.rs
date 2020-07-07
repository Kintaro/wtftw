extern crate libc;

use self::libc::{c_int, c_ulong};
use crate::config::GeneralConfig;
use std::fmt::{Debug, Error, Formatter};
use crate::window_manager::WindowManager;

pub type Window = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle(pub i32, pub i32, pub u32, pub u32);

impl Rectangle {
    pub fn is_inside(&self, x: i32, y: i32) -> bool {
        let &Rectangle(rx, ry, rw, rh) = self;

        x >= rx && x <= rx + rw as i32 && y >= ry && y <= ry + rh as i32
    }

    pub fn overlaps(&self, &Rectangle(bx, by, bw, bh): &Rectangle) -> bool {
        let &Rectangle(ax, ay, aw, ah) = self;
        !(bx >= ax + aw as i32
            || bx + bw as i32 <= ax
            || by >= ay + ah as i32
            || by + bh as i32 <= ay)
    }
}

#[derive(Clone, Copy, Debug)]
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyCommand {
    pub mask: KeyModifiers,
    pub key: u64,
}

impl KeyCommand {
    pub fn new(key: u64, mask: KeyModifiers) -> KeyCommand {
        KeyCommand { key, mask }
    }
}

impl Debug for KeyCommand {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:}", self.key)).unwrap();
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseCommand {
    pub mask: KeyModifiers,
    pub button: MouseButton,
}

impl Debug for MouseCommand {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:}", self.button)).unwrap();
        Ok(())
    }
}

impl MouseCommand {
    pub fn new(button: MouseButton, mask: KeyModifiers) -> MouseCommand {
        MouseCommand { button, mask }
    }
}

bitflags! {
    pub struct KeyModifiers : u32 {
        const NONEMASK    = (0 << 0);
        const SHIFTMASK   = (1 << 0);
        const LOCKMASK    = (1 << 1);
        const CONTROLMASK = (1 << 2);
        const MOD1MASK    = (1 << 3);
        const MOD2MASK    = (1 << 4);
        const MOD3MASK    = (1 << 5);
        const MOD4MASK    = (1 << 6);
        const MOD5MASK    = (1 << 7);
    }
}

pub type MouseButton = u32;
pub const BUTTON1: MouseButton = 1;
pub const BUTTON2: MouseButton = 2;
pub const BUTTON3: MouseButton = 3;
pub const BUTTON4: MouseButton = 4;
pub const BUTTON5: MouseButton = 5;

impl KeyModifiers {
    pub fn get_mask(&self) -> u32 {
        self.bits()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SizeHint {
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
}

#[derive(Clone, Copy, Debug)]
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
    ButtonPressed(Window, Window, MouseCommand, u32, u32),
    ButtonReleased,
    MouseMotion(u32, u32),
    KeyPressed(Window, KeyCommand),
    ClientMessageEvent(Window, c_ulong, c_int, [i32; 5]),
    PropertyMessageEvent(bool, Window, c_ulong),
    /// The underlying event by xlib or wayland is unknown
    /// and can be ignored.
    UnknownEvent,
}

pub trait WindowSystem {
    fn get_string_from_keycode(&self, key: u32) -> String;
    fn get_keycode_from_string(&self, key: &str) -> u64;
    fn get_root(&self) -> Window;
    /// Retrieve geometry infos over all screens
    fn get_screen_infos(&self) -> Vec<Rectangle>;
    /// Get the number of physical displays
    fn get_number_of_screens(&self) -> usize;
    /// Get the width of the given physical screen
    fn get_display_width(&self, screen: usize) -> u32;
    /// Get the height of the given physical screen
    fn get_display_height(&self, screen: usize) -> u32;
    /// Get the given window's name
    fn get_window_name(&self, window: Window) -> String;
    fn get_class_name(&self, window: Window) -> String;
    // Get 'role' name of window
    fn get_role_name(&self, window: Window) -> String;
    /// Get a list of all windows
    fn get_windows(&self) -> Vec<Window>;
    /// Set the given window's border width
    fn set_window_border_width(&self, window: Window, border_width: u32);
    fn get_window_border_width(&self, window: Window) -> u32;
    /// Set the given window's border color
    fn set_window_border_color(&self, window: Window, border_color: u32);
    /// Resize the window to the given dimensions
    fn resize_window(&self, window: Window, width: u32, height: u32);
    /// Move the window's top left corner to the given coordinates
    fn move_window(&self, window: Window, x: i32, y: i32);
    /// Map the window to the screen and show it
    fn show_window(&self, window: Window);
    fn hide_window(&self, window: Window);
    fn focus_window(&self, window: Window, window_manager: &WindowManager);
    fn get_focused_window(&self) -> Window;
    fn configure_window(
        &self,
        window: Window,
        window_changes: WindowChanges,
        mask: u64,
        is_floating: bool,
    );
    /// Check if there are events pending
    fn event_pending(&self) -> bool;
    /// Get the next event from the queue
    fn get_event(&self) -> WindowSystemEvent;
    fn flush(&self);
    fn grab_keys(&self, keys: Vec<KeyCommand>);
    fn grab_button(&self, button: MouseCommand);
    fn remove_enter_events(&self);
    fn remove_motion_events(&self);
    fn get_partial_strut(&self, window: Window) -> Option<Vec<u64>>;
    fn get_strut(&self, window: Window) -> Option<Vec<u64>>;
    fn set_initial_properties(&self, window: Window);
    fn is_dock(&self, window: Window) -> bool;
    fn get_geometry(&self, window: Window) -> Rectangle;
    fn get_size_hints(&self, window: Window) -> SizeHint;
    fn restack_windows(&self, windows: Vec<Window>);
    fn close_client(&self, window: Window);
    fn kill_client(&self, window: Window);
    fn grab_pointer(&self);
    fn ungrab_pointer(&self);
    fn get_pointer(&self, window: Window) -> (u32, u32);
    fn warp_pointer(&self, window: Window, x: u32, y: u32);
    fn overrides_redirect(&self, window: Window) -> bool;
    fn update_server_state(&self, manager: &WindowManager);
    fn process_message(
        &self,
        window_manager: &WindowManager,
        config: &GeneralConfig,
        window: Window,
        atom: c_ulong,
    ) -> WindowManager;
}
