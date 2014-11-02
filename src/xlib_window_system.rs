extern crate libc;
extern crate xlib;

use self::libc::{ c_int, c_void };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::{
    Display,
    XSetWindowBorder,
    XSetWindowBorderWidth,
    XDefaultRootWindow,
    XDisplayWidth,
    XDisplayHeight,
    XMapRequestEvent,
    XMapWindow,
    XNextEvent,
    XOpenDisplay,
    XPending,
    XResizeWindow,
    XSelectInput
};

use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;

use window_system::{ WindowSystem, WindowSystemEvent };
use window_system::{
    WindowCreated,
    WindowDestroyed,
    UnknownEvent
};

pub struct XlibWindowSystem {
    display: *mut Display,
    event: *mut c_void
}

impl XlibWindowSystem {
    pub fn new() -> XlibWindowSystem {
        unsafe {
            let display = XOpenDisplay(null_mut());
            let root    = XDefaultRootWindow(display);

            XSelectInput(display, root, 0x1E0000i64);

            XlibWindowSystem {
                display: display,
                event: malloc(256)
            }
        }
    }
}

impl WindowSystem for XlibWindowSystem {
    fn get_display_width(&self, screen: uint) -> u32 {
        unsafe {
            XDisplayWidth(self.display, screen as i32) as u32
        }
    }

    fn get_display_height(&self, screen: uint) -> u32 {
        unsafe {
            XDisplayHeight(self.display, screen as i32) as u32
        }
    }

    fn set_window_border_width(&mut self, window: u64, border_width: uint) {
        unsafe {
            XSetWindowBorderWidth(self.display, window, border_width as u32); 
        }
    }

    fn set_window_border_color(&mut self, window: u64, border_color: uint) {
        unsafe {
            XSetWindowBorder(self.display, window, border_color as u64);   
        }
    }

    fn resize_window(&mut self, window: u64, width: u32, height: u32) {
        unsafe {
            XResizeWindow(self.display, window, width, height);
        }
    }

    fn show_window(&mut self, window: u64) {
        unsafe {
            XMapWindow(self.display, window);
        }
    }

    fn event_pending(&self) -> bool {
        unsafe {
            XPending(self.display) != 0
        }
    }

    fn get_event(&mut self) -> WindowSystemEvent {
        unsafe {
            XNextEvent(self.display, self.event);

            let event_type_ptr : *const c_int = transmute(self.event);
            let event_type = *event_type_ptr;

            match event_type {
                20 => {
                    let map_request_event_ptr : *const XMapRequestEvent = transmute(self.event);
                    let map_request_event = *map_request_event_ptr;
                    let window = map_request_event.window;
                    WindowCreated(window)
                },
                _  => UnknownEvent
            }
        }
    }
}
