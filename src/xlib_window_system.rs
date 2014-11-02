extern crate libc;
extern crate xlib;

use self::libc::{ c_char, c_int, c_void };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::{
    Display,
    Window,
    XCrossingEvent,
    XSetWindowBorder,
    XSetWindowBorderWidth,
    XDefaultRootWindow,
    XDefaultScreenOfDisplay,
    XDisplayWidth,
    XDisplayHeight,
    XEnterWindowEvent,
    XFetchName,
    XLeaveWindowEvent,
    XMapRequestEvent,
    XMapWindow,
    XMotionEvent,
    XMoveWindow,
    XNextEvent,
    XOpenDisplay,
    XPending,
    XResizeWindow,
    XRootWindowOfScreen,
    XSelectInput,
    XSync
};

use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::raw::c_str_to_static_slice;

use window_system::{ WindowSystem, WindowSystemEvent };
use window_system::{
    Enter,
    Leave,
    WindowCreated,
    WindowDestroyed,
    UnknownEvent
};

const KeyPress               : uint =  2;
const KeyRelease             : uint =  3;
const ButtonPress            : uint =  4;
const ButtonRelease          : uint =  5;
const MotionNotify           : uint =  6;
const EnterNotify            : uint =  7;
const LeaveNotify            : uint =  8;
const FocusIn                : uint =  9;
const FocusOut               : uint = 10;
const KeymapNotify           : uint = 11;
const Expose                 : uint = 12;
const GraphicsExpose         : uint = 13;
const NoExpose               : uint = 14;
const VisibilityNotify       : uint = 15;
const CreateNotify           : uint = 16;
const DestroyNotify          : uint = 17;
const UnmapNotify            : uint = 18;
const MapNotify              : uint = 19;
const MapRequest             : uint = 20;
const ReparentNotify         : uint = 21;
const ConfigureNotify        : uint = 22;
const ConfigureRequest       : uint = 23;
const GravityNotify          : uint = 24;
const ResizeRequest          : uint = 25;
const CirculateNotify        : uint = 26;
const CirculateRequest       : uint = 27;
const PropertyNotify         : uint = 28;
const SelectionClear         : uint = 29;
const SelectionRequest       : uint = 30;
const SelectionNotify        : uint = 31;
const ColormapNotify         : uint = 32;
const ClientMessage          : uint = 33;
const MappingNotify          : uint = 34;

pub struct XlibWindowSystem {
    display: *mut Display,
    root:    Window,
    event:   *mut c_void
}

impl XlibWindowSystem {
    pub fn new() -> XlibWindowSystem {
        unsafe {
            let display = XOpenDisplay(null_mut());
            let screen  = XDefaultScreenOfDisplay(display);
            let root    = XRootWindowOfScreen(screen);

            XSelectInput(display, root, 0x180030);
            XSync(display, 0);

            XlibWindowSystem {
                display: display,
                root:    root,
                event:   malloc(256)
            }
        }
    }

    fn get_event_as<T>(&self) -> &T {
        unsafe {
            let event_ptr : *const T = transmute(self.event);
            let ref event = *event_ptr;
            event
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

    fn get_window_name(&self, window: u64) -> String {
        if window == self.root { return String::from_str("root"); }
        unsafe {
            let mut name : *mut c_char = uninitialized();
            let mut name_ptr : *mut *mut c_char = &mut name;
            XFetchName(self.display, window, name_ptr);
            String::from_str(c_str_to_static_slice(transmute(*name_ptr)))
        }
    }

    fn set_window_border_width(&mut self, window: u64, border_width: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorderWidth(self.display, window, border_width as u32); 
        }
    }

    fn set_window_border_color(&mut self, window: u64, border_color: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorder(self.display, window, border_color as u64);   
        }
    }

    fn resize_window(&mut self, window: u64, width: u32, height: u32) {
        unsafe {
            XResizeWindow(self.display, window, width, height);
        }
    }

    fn move_window(&mut self, window: u64, x: u32, y: u32) {
        unsafe {
            XMoveWindow(self.display, window, x as i32, y as i32);
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
            XSync(self.display, 0);
            XNextEvent(self.display, self.event);
        }
        
        let event_type : c_int = *self.get_event_as();

        match event_type as uint {
            MapRequest => {
                let event : &XMapRequestEvent = self.get_event_as();
                unsafe { XSelectInput(self.display, event.window, 0x000030); }
                WindowCreated(event.window)
            },
            EnterNotify => {
                let event : &XEnterWindowEvent = self.get_event_as();
                Enter(event.window) 
            },
            LeaveNotify => {
                let event : &XLeaveWindowEvent = self.get_event_as();
                Leave(event.window) 
            },
            _  => {
                println!("Unknown event {}", event_type);
                UnknownEvent
            }
        }
    }
}
