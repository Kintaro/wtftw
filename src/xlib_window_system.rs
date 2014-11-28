extern crate libc;
extern crate xlib;
extern crate xinerama;

use self::libc::{ c_char, c_uchar, c_int, c_uint, c_void, c_ulong };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::*;
use self::xinerama::XineramaQueryScreens;

use std::os::env;
use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::from_c_str;
use std::slice::from_raw_buf;
use std::c_str::CString;

use window_system::*;

const KEYPRESS               : uint =  2;
const BUTTONPRESS            : uint =  4;
const ENTERNOTIFY            : uint =  7;
const LEAVENOTIFY            : uint =  8;
const DESTROYNOTIFY          : uint = 17;
const UNMAPNOTIFY            : uint = 18;
const MAPREQUEST             : uint = 20;
const CONFIGURENOTIFY        : uint = 22;
const CONFIGUREREQUEST       : uint = 23;
const CLIENTMESSAGE          : uint = 33;
const BADWINDOW              :  i32 =  3;

/// A custom error handler to prevent xlib from crashing the whole WM.
/// Necessary because a few events may call the error routine.
extern fn error_handler(_: *mut Display, _: *mut XErrorEvent) -> c_int {
    return 0;
}

/// The xlib interface. Holds a pointer to the display,
/// the root window's id and a generic event so
/// we don't have to allocate it every time.
pub struct XlibWindowSystem {
    display: *mut Display,
    root:    Window,
    event:   *mut c_void,
}

impl XlibWindowSystem {
    /// Creates a new xlib interface on the default display (i.e. ${DISPLAY})
    /// and creates a root window spanning all screens (including Xinerama).
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
            XSync(display, 1);

            XlibWindowSystem {
                display: display,
                root:    root,
                event:   malloc(256),
            }
        }
    }

    /// Cast our generic event to the desired type
    unsafe fn get_event_as<T>(&self) -> &T {
        &*(self.event as *const T)
    }

    fn get_property(&self, atom: Window, window: Window) -> Option<Vec<c_ulong>> {
        unsafe {
            let mut actual_type_return : Window = 0;
            let mut actual_format_return : c_int = 0;
            let mut nitems_return : c_ulong = 0;
            let mut bytes_after_return : c_ulong = 0;
            let mut prop_return : *mut c_uchar = uninitialized();

            let r = XGetWindowProperty(self.display, window, atom, 0, 0xFFFFFFFF, 0, 0,
                               &mut actual_type_return,
                               &mut actual_format_return,
                               &mut nitems_return,
                               &mut bytes_after_return,
                               &mut prop_return);

            if r != 0 {
                None
            } else {
                if actual_format_return == 0 {
                    None
                } else {
                    Some(from_raw_buf(&(prop_return as *const c_ulong), nitems_return as uint).iter()
                                .map(|&c| c)
                                .collect())
                }
            }
        }
    }

    fn get_property_from_string(&self, s: &str, window: Window) -> Option<Vec<c_ulong>> {
        unsafe { 
            let atom = XInternAtom(self.display, s.to_c_str().as_mut_ptr(), 0);
            self.get_property(atom, window)
        }
    }
}

impl WindowSystem for XlibWindowSystem {
    fn get_partial_strut(&self, window: Window) -> Option<Vec<c_ulong>> {
        self.get_property_from_string("_NET_WM_STRUT_PARTIAL", window)
    }

    fn get_strut(&self, window: Window) -> Option<Vec<c_ulong>> {
        self.get_property_from_string("_NET_WM_STRUT", window)
    }

    fn get_string_from_keycode(&self, key: u32) -> String {
        unsafe {
            let keysym = XKeycodeToKeysym(self.display, key as u8, 0);
            let keyname : *mut c_char = XKeysymToString(keysym);
            String::from_str(from_c_str(transmute(keyname)))
        }
    }

    fn get_keycode_from_string(&self, key: &str) -> u64 {
        unsafe {
            XStringToKeysym(key.to_c_str().as_mut_ptr())
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

            from_raw_buf(&screen_ptr, num as uint).iter().map(
                |&screen_info|
                    Rectangle(
                        screen_info.x_org as u32,
                        screen_info.y_org as u32,
                        screen_info.width as u32,
                        screen_info.height as u32)).collect()
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
            if XFetchName(self.display, window, &mut name) == BADWINDOW || name.is_null() {
                String::from_str("Unknown")
            } else {
                let string = CString::new(name as *const c_char, true);
                format!("{}", string)
            }
        }
    }

    fn get_class_name(&self, window: Window) -> String {
        unsafe {
            let mut class_hint : XClassHint = uninitialized();
            XGetClassHint(self.display, window, &mut class_hint);
            let string = CString::new(class_hint.res_class as *const c_char, true);
            format!("{}", string)
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
            from_raw_buf(&const_children, num_children as uint).iter()
                            .filter(|&&c| c != self.root)
                            .map(|&c| c)
                            .collect()
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
            CLIENTMESSAGE => {
                unsafe {
                    let event : &XClientMessageEvent = self.get_event_as();
                    WindowSystemEvent::ClientMessageEvent(event.window)
                }
            },
            CONFIGUREREQUEST => {
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

                WindowSystemEvent::ConfigurationRequest(event.window, window_changes, event.value_mask)
            },
            CONFIGURENOTIFY => {
                unsafe {
                    let event : &XConfigureEvent = self.get_event_as();
                    WindowSystemEvent::ConfigurationNotification(event.window)
                }
            },
            MAPREQUEST => {
                unsafe {
                    let event : &XMapRequestEvent = self.get_event_as();
                    XSelectInput(self.display, event.window, 0x420030);
                    WindowSystemEvent::WindowCreated(event.window)
                }
            },
            UNMAPNOTIFY => {
                unsafe {
                    let event : &XUnmapEvent = self.get_event_as();
                    WindowSystemEvent::WindowUnmapped(event.window, event.send_event > 0)
                }
            },
            DESTROYNOTIFY => {
                unsafe {
                    let event : &XDestroyWindowEvent = self.get_event_as();
                    WindowSystemEvent::WindowDestroyed(event.window)
                }
            },
            ENTERNOTIFY => {
                unsafe {
                    let event : &XEnterWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        WindowSystemEvent::Enter(event.window)
                    } else {
                        WindowSystemEvent::UnknownEvent
                    }
                }
            },
            LEAVENOTIFY => {
                unsafe {
                    let event : &XLeaveWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        WindowSystemEvent::Leave(event.window)
                    } else {
                        WindowSystemEvent::UnknownEvent
                    }
                }
            },
            BUTTONPRESS => {
                unsafe {
                    let event : &XButtonEvent = self.get_event_as();
                    WindowSystemEvent::ButtonPressed(event.window, event.state, event.button,
                                  event.x_root as u32, event.y_root as u32)
                }
            },
            KEYPRESS => {
                unsafe {
                    let event : XKeyEvent = *self.get_event_as();
                    let key = KeyCommand {
                        key: XKeycodeToKeysym(self.display, event.keycode as u8, 0),
                        mask: KeyModifiers::from_bits(0xEF & event.state as u32).unwrap()
                    };
                    debug!("key pressed: {} with mask {}", key.key, key.mask);
                    WindowSystemEvent::KeyPressed(event.window, key)
                }
            },
            _  => {
                WindowSystemEvent::UnknownEvent
            }
        }
    }

    fn grab_keys(&self, keys: Vec<KeyCommand>) {
        for key in keys.iter() {
            unsafe {
                debug!("grabbing key {}", key);
                XGrabKey(self.display, XKeysymToKeycode(self.display, key.key) as i32,
                         key.mask.get_mask(), self.root, 1, 1, 1);
                XGrabKey(self.display, XKeysymToKeycode(self.display, key.key) as i32,
                         key.mask.get_mask() | 0x10, self.root, 1, 1, 1);
            }
        }
    }

    fn remove_enter_events(&self) {
        unsafe {
            let event : *mut c_void = malloc(256);
            XSync(self.display, 0);
            while XCheckMaskEvent(self.display, 16, event) != 0 { }
        }
    }
}
