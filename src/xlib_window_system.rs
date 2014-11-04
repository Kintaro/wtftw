#![allow(non_upper_case_globals)]
#![allow(dead_code)]
extern crate libc;
extern crate xlib;
extern crate xinerama;

use self::libc::{ c_char, c_int, c_uint, c_void };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::{
    Display,
    Window,
    XConfigureRequestEvent,
    XButtonEvent,
    XSetWindowBorder,
    XSetWindowBorderWidth,
    XDefaultScreenOfDisplay,
    XDestroyWindowEvent,
    XDisplayWidth,
    XDisplayHeight,
    XEnterWindowEvent,
    XErrorEvent,
    XFetchName,
    XLeaveWindowEvent,
    XMapRequestEvent,
    XMapWindow,
    XMoveWindow,
    XNextEvent,
    XOpenDisplay,
    XPending,
    XQueryTree,
    XResizeWindow,
    XRootWindowOfScreen,
    XScreenCount,
    XSelectInput,
    XSetErrorHandler,
    XSync,
    XUnmapEvent
};
use self::xinerama::{
    XineramaQueryScreens,
};

use std::os::env;
use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::raw::c_str_to_static_slice;
use std::slice::raw::buf_as_slice;

use window_system::{ Rectangle, WindowSystem, WindowSystemEvent };
use window_system::{
    ConfigurationRequest,
    Enter,
    Leave,
    WindowCreated,
    WindowDestroyed,
    WindowUnmapped,
    ButtonPressed,
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

extern fn error_handler(display: *mut Display, event: *mut XErrorEvent) -> c_int {
    return 0;
}

pub struct XlibWindowSystem {
    display: *mut Display,
    root:    Window,
    event:   *mut c_void
}

impl XlibWindowSystem {
    pub fn new() -> XlibWindowSystem {
        unsafe {
            let display = XOpenDisplay(null_mut());

            if display == null_mut() {
                error!("No display found at {}",
                    env().iter()
                       .find(|&&(ref d, _)| *d == String::from_str("DISPLAY"))
                       .map(|&(_, ref v)| v.clone())
                       .unwrap());
                panic!("Exiting");
            }

            let screen  = XDefaultScreenOfDisplay(display);
            let root    = XRootWindowOfScreen(screen);

            XSetErrorHandler(error_handler as *mut u8);

            XSelectInput(display, root, 0x180034);
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
    fn get_screen_infos(&self) -> Vec<Rectangle> {
        unsafe {
            let mut num : c_int = 0;
            let screen_ptr = XineramaQueryScreens(self.display, &mut num);

            // If xinerama is not active, just return the default display
            // dimensions and "emulate" xinerama.
            if num == 0 {
                return vec!(Rectangle(0, 0, 
                                      self.get_display_width(0) as uint, 
                                      self.get_display_height(0) as uint));
            }
            
            buf_as_slice(screen_ptr, num as uint, |x| {
                let mut result : Vec<Rectangle> = Vec::new();
                for &screen_info in x.iter() {
                    result.push(Rectangle(
                            screen_info.x_org as uint,
                            screen_info.y_org as uint,
                            screen_info.width as uint,
                            screen_info.height as uint));
                }
                result
            })
        }
    }

    fn get_number_of_screens(&self) -> uint {
        unsafe {
            XScreenCount(self.display) as uint
        }
    }

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

    fn get_window_name(&self, window: Window) -> String {
        if window == self.root { return String::from_str("root"); }
        unsafe {
            let mut name : *mut c_char = uninitialized();
            XFetchName(self.display, window, &mut name);
            String::from_str(c_str_to_static_slice(transmute(name)))
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        unsafe {
            let mut unused = 0u64;
            let mut children : *mut u64 = uninitialized();
            let children_ptr : *mut *mut u64 = &mut children;
            let mut num_children : c_uint = 0;
            XQueryTree(self.display, self.root, &mut unused, &mut unused, children_ptr, &mut num_children);
            let const_children : *const u64 = children as *const u64;
            buf_as_slice(const_children, num_children as uint, |x| 
                         x.to_vec().iter()
                            .filter(|&&c| c != self.root)
                            .map(|&c| c)
                            .collect())
        }
    }

    fn set_window_border_width(&mut self, window: Window, border_width: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorderWidth(self.display, window, border_width as u32); 
        }
    }

    fn set_window_border_color(&mut self, window: Window, border_color: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorder(self.display, window, border_color as Window);   
        }
    }

    fn resize_window(&mut self, window: Window, width: uint, height: uint) {
        unsafe {
            XResizeWindow(self.display, window, width as u32, height as u32);
        }
    }

    fn move_window(&mut self, window: Window, x: uint, y: uint) {
        unsafe {
            XMoveWindow(self.display, window, x as i32, y as i32);
        }
    }

    fn show_window(&mut self, window: Window) {
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
            //XSync(self.display, 0);
            XNextEvent(self.display, self.event);
        }
        
        let event_type : c_int = *self.get_event_as();

        match event_type as uint {
            ConfigureRequest => {
                let event : &XConfigureRequestEvent = self.get_event_as();
                ConfigurationRequest(event.window)
            },
            MapRequest => {
                let event : &XMapRequestEvent = self.get_event_as();
                unsafe { XSelectInput(self.display, event.window, 0x000030); }
                WindowCreated(event.window)
            },
            UnmapNotify => {
                let event : &XUnmapEvent = self.get_event_as();
                WindowUnmapped(event.window, event.send_event > 0)
            },
            DestroyNotify => {
                let event : &XDestroyWindowEvent = self.get_event_as();
                WindowDestroyed(event.window)
            },
            EnterNotify => {
                let event : &XEnterWindowEvent = self.get_event_as();
                if event.detail != 2 {
                    Enter(event.window) 
                } else {
                    UnknownEvent
                }
            },
            LeaveNotify => {
                let event : &XLeaveWindowEvent = self.get_event_as();
                if event.detail != 2 {
                    Leave(event.window) 
                } else {
                    UnknownEvent
                }
            },
            ButtonPress => {
                let event : &XButtonEvent = self.get_event_as();
                ButtonPressed(event.window, event.state as uint, event.button as uint, 
                              event.x_root as uint, event.y_root as uint)
            }
            _  => {
                UnknownEvent
            }
        }
    }
}
