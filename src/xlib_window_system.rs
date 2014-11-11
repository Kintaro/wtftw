#![allow(non_upper_case_globals)]
#![allow(dead_code)]
extern crate libc;
extern crate xlib;
extern crate xinerama;

use self::libc::{ c_char, c_int, c_uint, c_void };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::*;
use self::xinerama::XineramaQueryScreens;

use std::os::env;
use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::raw::c_str_to_static_slice;
use std::slice::raw::buf_as_slice;

use window_system::*;

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

extern fn error_handler(_: *mut Display, _: *mut XErrorEvent) -> c_int {
    return 0;
}

#[deriving(Clone)]
pub struct XlibWindowSystem {
    display: *mut Display,
    screen:  *mut Screen,
    root:    Window,
    event:   *mut c_void,
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

            XSelectInput(display, root, 0x1A0035);
            XSync(display, 0);

            XlibWindowSystem {
                display: display,
                screen:  screen,
                root:    root,
                event:   malloc(256),
            }
        }
    }

    unsafe fn get_event_as<T>(&self) -> &T {
        &*(self.event as *const T)
    }
}

impl WindowSystem for XlibWindowSystem {
    fn get_string_from_keycode(&self, key: u32) -> String {
        unsafe {
            let keysym = XKeycodeToKeysym(self.display, key as u8, 0);
            let keyname : *mut c_char = XKeysymToString(keysym);
            String::from_str(c_str_to_static_slice(transmute(keyname)))
        } 
    }

    fn get_keycode_from_string(&self, key: &String) -> u32 {
        unsafe {
            let keysym = XStringToKeysym(key.to_c_str().as_mut_ptr());
            XKeysymToKeycode(self.display, keysym) as u32
        }
    }

    fn get_root(&self) -> Window {
        self.root
    }

    fn get_screen_infos(&self) -> Vec<Rectangle> {
        unsafe {
            let mut num : c_int = 0;
            let screen_ptr = XineramaQueryScreens(self.display, &mut num);

            // If xinerama is not active, just return the default display
            // dimensions and "emulate" xinerama.
            if num == 0 {
                return vec!(Rectangle(0, 0, 
                                      self.get_display_width(0), 
                                      self.get_display_height(0)));
            }
            
            buf_as_slice(screen_ptr, num as uint, |x| {
                let mut result : Vec<Rectangle> = Vec::new();
                for &screen_info in x.iter() {
                    result.push(Rectangle(
                            screen_info.x_org as u32,
                            screen_info.y_org as u32,
                            screen_info.width as u32,
                            screen_info.height as u32));
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

    fn set_window_border_width(&self, window: Window, border_width: u32) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorderWidth(self.display, window, border_width); 
        }
    }

    fn set_window_border_color(&self, window: Window, border_color: u32) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorder(self.display, window, border_color as u64);   
        }
    }

    fn resize_window(&self, window: Window, width: u32, height: u32) {
        unsafe {
            XResizeWindow(self.display, window, width, height);
        }
    }

    fn move_window(&self, window: Window, x: u32, y: u32) {
        unsafe {
            XMoveWindow(self.display, window, x as i32, y as i32);
        }
    }

    fn show_window(&self, window: Window) {
        unsafe {
            XMapWindow(self.display, window);
        }
    }

    fn hide_window(&self, window: Window) {
        unsafe {
            XSelectInput(self.display, window, 0x1A0030);
            XUnmapWindow(self.display, window);
            XSelectInput(self.display, window, 0x1A0030);
            XIconifyWindow(self.display, window, 0);
        }
    }

    fn focus_window(&self, window: Window) {
        unsafe {
            XSetInputFocus(self.display, window, 1, 0);
        }
    }

    fn get_focused_window(&self) -> Window {
        unsafe {
            let mut window = 0;
            let mut tmp = 0;

            XGetInputFocus(self.display, &mut window, &mut tmp) as Window
        }
    }

    fn configure_window(&self, window: Window, window_changes: WindowChanges, mask: u64) {
        unsafe {
            let mut xlib_window_changes = XWindowChanges {
                x: window_changes.x as i32,
                y: window_changes.y as i32,
                width: window_changes.width as i32,
                height: window_changes.height as i32,
                border_width: window_changes.border_width as i32,
                sibling: window_changes.sibling,
                stack_mode: window_changes.stack_mode as i32
            };
            XConfigureWindow(self.display, window, mask as u32, &mut xlib_window_changes);
        }
    }

    fn flush(&self) {
        unsafe {
            XFlush(self.display);
        }
    }

    fn event_pending(&self) -> bool {
        unsafe {
            XPending(self.display) != 0
        }
    }

    fn get_event(&self) -> WindowSystemEvent {
        unsafe {
            XNextEvent(self.display, self.event);
        }

        let event_type : c_int = unsafe { *self.get_event_as() };

        match event_type as uint {
            ClientMessage => {
                unsafe {
                    let event : &XClientMessageEvent = self.get_event_as();
                    ClientMessageEvent(event.window)
                }
            },
            ConfigureRequest => {
                let event : &XConfigureRequestEvent = unsafe { self.get_event_as() };
                let window_changes = WindowChanges {
                    x: event.x as u32,
                    y: event.y as u32,
                    width: event.width as u32,
                    height: event.height as u32,
                    border_width: event.border_width as u32,
                    sibling: event.above as Window,
                    stack_mode: event.detail as u32
                };

                ConfigurationRequest(event.window, window_changes, event.value_mask)
            },
            ConfigureNotify => {
                unsafe {
                    let event : &XConfigureEvent = self.get_event_as();
                    ConfigurationNotification(event.window)
                }
            },
            MapRequest => {
                unsafe {
                    let event : &XMapRequestEvent = self.get_event_as();
                    XSelectInput(self.display, event.window, 0x420030);
                    debug!("map request {}", self.get_window_name(event.window));
                    WindowCreated(event.window)
                }
            },
            UnmapNotify => {
                unsafe {
                    let event : &XUnmapEvent = self.get_event_as();
                    WindowUnmapped(event.window, event.send_event > 0)
                }
            },
            DestroyNotify => {
                unsafe {
                    let event : &XDestroyWindowEvent = self.get_event_as();
                    WindowDestroyed(event.window)
                }
            },
            EnterNotify => {
                unsafe {
                    let event : &XEnterWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        Enter(event.window) 
                    } else {
                        UnknownEvent
                    }
                }
            },
            LeaveNotify => {
                unsafe {
                    let event : &XLeaveWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        Leave(event.window) 
                    } else {
                        UnknownEvent
                    }
                }
            },
            ButtonPress => {
                unsafe {
                    let event : &XButtonEvent = self.get_event_as();
                    ButtonPressed(event.window, event.state, event.button, 
                                  event.x_root as u32, event.y_root as u32)
                }
            },
            KeyPress => {
                unsafe {
                    let event : XKeyEvent = *self.get_event_as();
                    let key = KeyCommand { 
                        key: self.get_string_from_keycode(event.keycode),
                        mask: KeyModifiers::from_bits(0xEF & event.state as u32).unwrap()
                    };
                    debug!("key pressed: {} with mask {}", key.key, key.mask);
                    KeyPressed(event.window, key)
                }
            },
            _  => {
                UnknownEvent
            }
        }
    }

    fn grab_keys(&self, keys: Vec<KeyCommand>) {
        for key in keys.iter() {
            unsafe {
                debug!("grabbing key {}", key);
                XGrabKey(self.display, self.get_keycode_from_string(&key.key) as i32, 
                         key.mask.get_mask(), self.root, 1, 1, 1); 
                XGrabKey(self.display, self.get_keycode_from_string(&key.key) as i32, 
                         key.mask.get_mask() | 0x10, self.root, 1, 1, 1); 
            }
        }
    }
}
