#[macro_use]

extern crate log;
extern crate libc;
extern crate xlib;
extern crate xinerama;
extern crate wtftw_core;

use std::borrow::ToOwned;
use libc::{ c_char, c_uchar, c_int, c_uint, c_void, c_long, c_ulong };
use libc::funcs::c95::stdlib::malloc;
use xlib::{
    Display,
    PMinSize,
    PMaxSize,
    XButtonEvent,
    XChangeProperty,
    XCheckMaskEvent,
    XClientMessageEvent,
    XConfigureEvent,
    XConfigureRequestEvent,
    XConfigureWindow,
    XDefaultScreenOfDisplay,
    XDestroyWindowEvent,
    XDisplayWidth,
    XDisplayHeight,
    XEnterWindowEvent,
    XErrorEvent,
    XFetchName,
    XFlush,
    XGetInputFocus,
    XGetWindowAttributes,
    XGetWindowProperty,
    XGetWMNormalHints,
    XGetWMProtocols,
    XGrabButton,
    XGrabKey,
    XGrabPointer,
    XInternAtom,
    XKeycodeToKeysym,
    XKeyEvent,
    XKeysymToKeycode,
    XKeysymToString,
    XKillClient,
    XLeaveWindowEvent,
    XMapRequestEvent,
    XMapWindow,
    XMotionEvent,
    XMoveWindow,
    XNextEvent,
    XOpenDisplay,
    XPending,
    XQueryPointer,
    XQueryTree,
    XResizeWindow,
    XRestackWindows,
    XRootWindowOfScreen,
    XScreenCount,
    XSendEvent,
    XSelectInput,
    XSetErrorHandler,
    XSetInputFocus,
    XSetWindowBorder,
    XSetWindowBorderWidth,
    XSizeHints,
    XStringToKeysym,
    XSync,
    XUngrabButton,
    XUngrabPointer,
    XUnmapEvent,
    XUnmapWindow,
    XWarpPointer,
    XWindowAttributes,
    XWindowChanges,
};
use xinerama::XineramaQueryScreens;
use xinerama::XineramaScreenInfo;

use std::env::vars;
use std::str;
use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::from_utf8;
use std::slice::from_raw_parts;
use std::ffi::CString;
use std::ffi::CStr;

use wtftw_core::window_system::*;
use wtftw_core::window_manager::*;

const KEYPRESS               : usize =  2;
const BUTTONPRESS            : usize =  4;
const BUTTONRELEASE          : usize =  5;
const MOTIONOTIFY            : usize =  6;
const ENTERNOTIFY            : usize =  7;
const LEAVENOTIFY            : usize =  8;
const DESTROYNOTIFY          : usize = 17;
const UNMAPNOTIFY            : usize = 18;
const MAPREQUEST             : usize = 20;
const CONFIGURENOTIFY        : usize = 22;
const CONFIGUREREQUEST       : usize = 23;
const CLIENTMESSAGE          : usize = 33;
const BADWINDOW              :  i32 =  3;

/// A custom error handler to prevent xlib from crashing the whole WM.
/// Necessary because a few events may call the error routine.
extern fn error_handler(_: *mut Display, _: *mut XErrorEvent) -> c_int {
    return 0;
}

/// The xlib interface. Holds a pointer to the display,
/// the root window's id and a generic event so
/// we don't have to allocate it every time.
//#[derive(Clone, Copy)]
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
                    vars()
                       .find(|&(ref d, _)| *d == "DISPLAY".to_owned())
                       .map(|(_, v)| v)
                       .unwrap());
                panic!("Exiting");
            }

            let screen  = XDefaultScreenOfDisplay(display);
            let root    = XRootWindowOfScreen(screen);

            XSetErrorHandler(error_handler as *mut u8);

            XSelectInput(display, root, 0x1A0034);
            XSync(display, 0);

            XUngrabButton(display, 0, 0x8000, root);

            let res = XlibWindowSystem {
                display: display,
                root:    root as u64,
                event:   malloc(256),
            };

            let name = (*CString::new(&b"wtftw"[..]).unwrap()).as_ptr();

            let wmcheck = res.get_atom("_NET_SUPPORTING_WM_CHECK");
            let wmname = res.get_atom("_NET_WM_NAME");
            let utf8 = res.get_atom("UTF8_STRING");
            let xa_window = res.get_atom("XA_WINDOW");

            let mut root_cpy = root;
            let root_ptr : *mut Window = &mut root_cpy;
            XChangeProperty(display, root, wmcheck, xa_window, 32, 0, root_ptr as *mut c_uchar, 1);
            XChangeProperty(display, root, wmname, utf8, 8, 0, name as *mut c_uchar, 5);

            res
        }
    }

    /// Cast our generic event to the desired type
    unsafe fn get_event_as<T>(&self) -> &T {
        &*(self.event as *const T)
    }

    fn get_property(&self, atom: Window, window: Window) -> Option<Vec<u64>> {
        unsafe {
            let mut actual_type_return : c_ulong = 0;
            let mut actual_format_return : c_int = 0;
            let mut nitems_return : c_ulong = 0;
            let mut bytes_after_return : c_ulong = 0;
            let mut prop_return : *mut c_uchar = uninitialized();

            let r = XGetWindowProperty(self.display, window as c_ulong, atom as c_ulong, 0, 0xFFFFFFFF, 0, 0,
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
                    Some(from_raw_parts(prop_return as *const c_ulong, nitems_return as usize).iter()
                                .map(|&c| c as u64)
                                .collect())
                }
            }
        }
    }

    fn get_property_from_string(&self, s: &str, window: Window) -> Option<Vec<u64>> {
        unsafe {
            match CString::new(s.as_bytes()) {
                Ok(b) => {
                    let atom = XInternAtom(self.display, b.as_ptr() as *mut i8, 0);
                    self.get_property(atom as u64, window)
                },
                _     => None
            }
        }
    }

    fn get_atom(&self, s: &str) -> u64 {
        unsafe {
            match CString::new(s) {
                Ok(b) => XInternAtom(self.display, b.as_ptr() as *mut i8, 0) as u64,
                _     => panic!("Invalid atom! {}", s)
            }
        }
    }

    fn get_protocols(&self, window: Window) -> Vec<u64> {
        unsafe {
            let mut protocols : *mut c_ulong = uninitialized();
            let mut num = 0;
            XGetWMProtocols(self.display, window as c_ulong, &mut protocols, &mut num);
            from_raw_parts(&(protocols as *const c_ulong), num as usize).iter()
                .map(|&c| c as u64)
                .collect::<Vec<_>>()
        }
    }

    fn change_property(&self, window: Window, property: u64, typ: u64, mode: c_int, dat: &mut [c_ulong]) {
        unsafe {
            let ptr : *mut u8 = transmute(dat.as_mut_ptr());
            XChangeProperty(self.display, window as c_ulong, property as c_ulong, typ as c_ulong, 32, mode, ptr, 2);
        }
    }

    fn set_button_grab(&self, grab: bool, window: Window) {
        if grab {
            debug!("grabbing mouse buttons for {}", window);
            for &button in (vec!(1, 2, 3)).iter() {
                unsafe { XGrabButton(self.display, button, 0x8000, window as c_ulong, 0, 4, 1, 0, 0, 0); }
            }
        } else {
            debug!("ungrabbing mouse buttons for {}", window);
            unsafe { XUngrabButton(self.display, 0, 0x8000, window as c_ulong); }
        }
    }

    fn set_focus(&self, window: Window, window_manager: &WindowManager) {
        debug!("setting focus to {}", window);
        for &other_window in window_manager.workspaces.visible_windows().iter() {
            self.set_button_grab(true, other_window);
        }
        if window != self.root {
            self.set_button_grab(false, window);
        }
    }
}

impl WindowSystem for XlibWindowSystem {
    fn get_partial_strut(&self, window: Window) -> Option<Vec<u64>> {
        self.get_property_from_string("_NET_WM_STRUT_PARTIAL", window)
    }

    fn get_strut(&self, window: Window) -> Option<Vec<u64>> {
        self.get_property_from_string("_NET_WM_STRUT", window)
    }

    fn is_dock(&self, window: Window) -> bool {
        let dock = self.get_atom("_NET_WM_WINDOW_TYPE_DOCK");
        let desk = self.get_atom("_NET_WM_WINDOW_TYPE_DESKTOP");

        if let Some(rs) =  self.get_property_from_string("_NET_WM_WINDOW_TYPE", window) {
            rs.iter().any(|&x| x == dock || x == desk)
        } else {
            false
        }
    }

    fn get_string_from_keycode(&self, key: u32) -> String {
        unsafe {
            let keysym = XKeycodeToKeysym(self.display, key as u8, 0);
            let keyname : *mut c_char = XKeysymToString(keysym);

            match from_utf8(CStr::from_ptr(transmute(keyname)).to_bytes()) {
                Ok(x) => x.to_owned(),
                _     => panic!("Invalid keycode!")
            }
        }
    }

    fn get_keycode_from_string(&self, key: &str) -> u64 {
        unsafe {
            match CString::new(key.as_bytes()) {
                Ok(b) => XStringToKeysym(b.as_ptr() as *mut i8) as u64,
                _     => panic!("Invalid key string!")
            }
        }
    }

    fn get_root(&self) -> Window {
        self.root
    }

    fn get_screen_infos(&self) -> Vec<Rectangle> {
        unsafe {
            let mut num : c_int = 0;
            let screen_ptr : *const XineramaScreenInfo = XineramaQueryScreens(self.display, &mut num);

            // If xinerama is not active, just return the default display
            // dimensions and "emulate" xinerama.
            if num == 0 {
                return vec!(Rectangle(0, 0,
                                      self.get_display_width(0),
                                      self.get_display_height(0)));
            }

            let screens = from_raw_parts(screen_ptr, num as usize).to_vec();
            screens.into_iter().map(
                |s| {
                    Rectangle(
                        s.x_org as i32,
                        s.y_org as i32,
                        s.width as u32,
                        s.height as u32)}).collect()
        }
    }

    fn get_number_of_screens(&self) -> usize {
        unsafe {
            XScreenCount(self.display) as usize
        }
    }

    fn get_display_width(&self, screen: usize) -> u32 {
        unsafe {
            XDisplayWidth(self.display, screen as i32) as u32
        }
    }

    fn get_display_height(&self, screen: usize) -> u32 {
        unsafe {
            XDisplayHeight(self.display, screen as i32) as u32
        }
    }

    fn get_window_name(&self, window: Window) -> String {
        if window == self.root { return "root".to_owned(); }
        unsafe {
            let mut name : *mut c_char = uninitialized();
            if XFetchName(self.display, window as c_ulong, &mut name) == BADWINDOW || name.is_null() {
                "Unknown".to_owned()
            } else {
                str::from_utf8_unchecked(CStr::from_ptr(name as *const c_char).to_bytes()).to_owned()
            }
        }
    }

    fn get_class_name(&self, _: Window) -> String {
        //unsafe {
            //let mut class_hint : XClassHint = uninitialized();
            //let result = if XGetClassHint(self.display, window as c_ulong, &mut class_hint) != 0 || class_hint.res_class.is_null() {
                "unknown".to_owned()
            //} else {
                //debug!("getting class name");
                //String::from_str(str::from_utf8_unchecked(ffi::c_str_to_bytes(&(class_hint.res_class as *const c_char))))
            //};

            //debug!("class name is {}", result);

            //result
        //}
    }

    fn get_windows(&self) -> Vec<Window> {
        unsafe {
            let mut unused : c_ulong = 0;
            let mut children : *mut c_ulong = uninitialized();
            let children_ptr : *mut *mut c_ulong = &mut children;
            let mut num_children : c_uint = 0;
            XQueryTree(self.display, self.root as c_ulong, &mut unused, &mut unused, children_ptr, &mut num_children);
            let const_children : *const u64 = children as *const u64;
            from_raw_parts(const_children, num_children as usize).iter()
                            .filter(|&&c| c != self.root)
                            .map(|c| *c)
                            .collect()
        }
    }

    fn set_window_border_width(&self, window: Window, border_width: u32) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorderWidth(self.display, window as c_ulong, border_width);
        }
    }

    fn get_window_border_width(&self, window: Window) -> u32 {
        unsafe {
            let mut attributes : XWindowAttributes = uninitialized();
            XGetWindowAttributes(self.display, window as c_ulong, &mut attributes);
            attributes.border_width as u32
        }
    }

    fn set_window_border_color(&self, window: Window, border_color: u32) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorder(self.display, window as c_ulong, border_color as c_ulong);
        }
    }

    fn resize_window(&self, window: Window, width: u32, height: u32) {
        unsafe {
            XResizeWindow(self.display, window as c_ulong, width, height);
        }
    }

    fn move_window(&self, window: Window, x: i32, y: i32) {
        unsafe {
            XMoveWindow(self.display, window as c_ulong, x, y);
        }
    }

    fn set_initial_properties(&self, window: Window) {
        unsafe {
            let atom = self.get_atom("WM_STATE");
            self.change_property(window as u64, atom, atom, 0, &mut [3, 0]);
            XSelectInput(self.display, window as c_ulong, 0x420010);
        }

    }

    fn show_window(&self, window: Window) {
        unsafe {
            let atom = self.get_atom("WM_STATE");
            self.change_property(window, atom, atom, 0, &mut [1, 0]);
            XMapWindow(self.display, window as c_ulong);
        }
    }

    fn hide_window(&self, window: Window) {
        unsafe {
            XSelectInput(self.display, window as c_ulong, 0x400010);
            XUnmapWindow(self.display, window as c_ulong);
            XSelectInput(self.display, window as c_ulong, 0x420010);
            let atom = self.get_atom("WM_STATE");
            self.change_property(window as u64, atom, atom, 0, &mut [3, 0]);

        }
    }

    fn focus_window(&self, window: Window, window_manager: &WindowManager) {
        unsafe {
            self.set_focus(window, window_manager);
            XSetInputFocus(self.display, window as c_ulong, 1, 0);
        }
    }

    fn get_focused_window(&self) -> Window {
        unsafe {
            let mut window = 0;
            let mut tmp = 0;

            XGetInputFocus(self.display, &mut window, &mut tmp) as Window
        }
    }

    fn configure_window(&self, window: Window, window_changes: WindowChanges, mask: u64, is_floating: bool) {
        unsafe {
            let result = if is_floating {
                let mut xlib_window_changes = XWindowChanges {
                    x: window_changes.x as i32,
                    y: window_changes.y as i32,
                    width: window_changes.width as i32,
                    height: window_changes.height as i32,
                    border_width: window_changes.border_width as i32,
                    sibling: window_changes.sibling as c_ulong,
                    stack_mode: window_changes.stack_mode as i32
                };
                XConfigureWindow(self.display, window as c_ulong, mask as u32, &mut xlib_window_changes);
            } else {
                let Rectangle(x, y, w, h) = self.get_geometry(window);

                let mut attributes : XWindowAttributes = uninitialized();
                XGetWindowAttributes(self.display, window as c_ulong, &mut attributes);

                let mut event : XConfigureEvent = uninitialized();


                event._type = CONFIGURENOTIFY as i32;
                event.x = attributes.x;
                event.y = attributes.y;
                event.width = attributes.width;
                event.height = attributes.height;
                event.border_width = attributes.border_width;
                event.above = 0;
                event.override_redirect = attributes.override_redirect;

                debug!("sending configure notification for window {}: ({}, {}) {}x{} redirect: {}",
                       window, x, y, w, h, attributes.override_redirect);
                let event_ptr : *mut XConfigureEvent = &mut event;
                XSendEvent(self.display, window as c_ulong, 0, 0, (event_ptr as *mut c_void));
            };

            XSync(self.display, 0);
            result
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

        match event_type as usize {
            CLIENTMESSAGE => {
                unsafe {
                    let event : &XClientMessageEvent = self.get_event_as();
                    WindowSystemEvent::ClientMessageEvent(event.window as u64)
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

                WindowSystemEvent::ConfigurationRequest(event.window as u64, window_changes, event.value_mask as u64)
            },
            CONFIGURENOTIFY => {
                unsafe {
                    let event : &XConfigureEvent = self.get_event_as();
                    WindowSystemEvent::ConfigurationNotification(event.window as u64)
                }
            },
            MAPREQUEST => {
                unsafe {
                    let event : &XMapRequestEvent = self.get_event_as();
                    WindowSystemEvent::WindowCreated(event.window as u64)
                }
            },
            UNMAPNOTIFY => {
                unsafe {
                    let event : &XUnmapEvent = self.get_event_as();
                    WindowSystemEvent::WindowUnmapped(event.window as u64, event.send_event > 0)
                }
            },
            DESTROYNOTIFY => {
                unsafe {
                    let event : &XDestroyWindowEvent = self.get_event_as();
                    WindowSystemEvent::WindowDestroyed(event.window as u64)
                }
            },
            ENTERNOTIFY => {
                unsafe {
                    let event : &XEnterWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        WindowSystemEvent::Enter(event.window as u64)
                    } else {
                        WindowSystemEvent::UnknownEvent
                    }
                }
            },
            LEAVENOTIFY => {
                unsafe {
                    let event : &XLeaveWindowEvent = self.get_event_as();
                    if event.detail != 2 {
                        WindowSystemEvent::Leave(event.window as u64)
                    } else {
                        WindowSystemEvent::UnknownEvent
                    }
                }
            },
            BUTTONPRESS => {
                unsafe {
                    let event : &XButtonEvent = self.get_event_as();
                    let button = MouseCommand {
                        button: event.button,
                        mask: KeyModifiers::from_bits(0xEF & event.state as u32).unwrap()
                    };
                    WindowSystemEvent::ButtonPressed(event.window as u64, event.subwindow as u64, button,
                                  event.x_root as u32, event.y_root as u32)
                }
            },
            BUTTONRELEASE => {
                WindowSystemEvent::ButtonReleased
            },
            KEYPRESS => {
                unsafe {
                    let event : &XKeyEvent = self.get_event_as();
                    let key = KeyCommand {
                        key: XKeycodeToKeysym(self.display, event.keycode as u8, 0) as u64,
                        mask: KeyModifiers::from_bits(0xEF & event.state as u32).unwrap()
                    };
                    WindowSystemEvent::KeyPressed(event.window as u64, key)
                }
            },
            MOTIONOTIFY => {
                unsafe {
                    let event : &XMotionEvent = self.get_event_as();
                    WindowSystemEvent::MouseMotion(event.x_root as u32, event.y_root as u32)
                }
            },
            _  => {
                debug!("unknown event is {}", event_type);
                WindowSystemEvent::UnknownEvent
            }
        }
    }

    fn grab_keys(&self, keys: Vec<KeyCommand>) {
        for &key in keys.iter() {
            unsafe {
                XGrabKey(self.display, XKeysymToKeycode(self.display, key.key as c_ulong) as i32,
                         key.mask.get_mask(), self.root as c_ulong, 1, 1, 1);
                XGrabKey(self.display, XKeysymToKeycode(self.display, key.key as c_ulong) as i32,
                         key.mask.get_mask() | 0x10, self.root as c_ulong, 1, 1, 1);
            }
        }
    }

    fn grab_button(&self, button: MouseCommand) {
        unsafe {
            XGrabButton(self.display, button.button, button.mask.get_mask(),
                        self.root as c_ulong, 0, 4, 1, 0, 0, 0);
        }
    }

    fn grab_pointer(&self) {
        unsafe {
            XGrabPointer(self.display, self.root as c_ulong, 0, 0x48, 1, 1, 0, 0, 0);
        }
    }

    fn ungrab_pointer(&self) {
        unsafe {
            XUngrabPointer(self.display, 0);
        }
    }

    fn remove_enter_events(&self) {
        unsafe {
            let event : *mut c_void = malloc(256);
            XSync(self.display, 0);
            while XCheckMaskEvent(self.display, 16, event) != 0 { }
        }
    }

    fn remove_motion_events(&self) {
        unsafe {
            let event : *mut c_void = malloc(256);
            XSync(self.display, 0);
            while XCheckMaskEvent(self.display, 0x40, event) != 0 { }
        }
    }

    fn get_geometry(&self, window: Window) -> Rectangle {
        unsafe {
            let mut attributes : XWindowAttributes = uninitialized();
            XGetWindowAttributes(self.display, window as c_ulong, &mut attributes);

            Rectangle(attributes.x as i32, attributes.y as i32, attributes.width as u32, attributes.height as u32)
        }
    }

    fn get_size_hints(&self, window: Window) -> SizeHint {
        unsafe {
            let mut size_hint : XSizeHints = uninitialized();
            let mut tmp : c_long = 0;
            XGetWMNormalHints(self.display, window as c_ulong, &mut size_hint, &mut tmp);

            let min_size = if size_hint.flags & PMinSize == PMinSize {
                Some((size_hint.min_width as u32, size_hint.min_height as u32))
            } else {
                None
            };

            let max_size = if size_hint.flags & PMaxSize == PMaxSize {
                Some((size_hint.max_width as u32, size_hint.max_height as u32))
            } else {
                None
            };

            SizeHint { min_size: min_size, max_size: max_size }
        }
    }

    fn restack_windows(&self, w: Vec<Window>) {
        unsafe {
            let mut windows = w.iter().map(|&x| x as c_ulong).collect::<Vec<_>>();
            XRestackWindows(self.display, (&mut windows[..]).as_mut_ptr(), windows.len() as i32);
        }
    }

    fn kill_client(&self, window: Window) {
        unsafe {
            let wmdelete = self.get_atom("WM_DELETE_WINDOW");
            let wmprotocols = self.get_atom("WM_PROTOCOLS");
            let protocols = self.get_protocols(window);

            debug!("supported protocols: {:?} (wmdelete = {:?})", protocols, wmdelete);

            if protocols.iter().any(|&x| x == wmdelete) {
                let mut event = XClientMessageEvent {
                    _type: 33,
                    serial: 0,
                    send_event: 0,
                    display: null_mut(),
                    window: window as c_ulong,
                    message_type: wmprotocols as c_ulong,
                    format: 32,
                    data: [((wmdelete & 0xFFFFFFFF00000000) >> 32) as i32,
                        (wmdelete & 0xFFFFFFFF) as i32, 0, 0, 0]
                };
                let event_pointer : *mut XClientMessageEvent = &mut event;
                XSendEvent(self.display, window as c_ulong, 0, 0, (event_pointer as *mut c_void));
            } else {
                XKillClient(self.display, window as c_ulong);
            }
        }
    }

    fn get_pointer(&self, window: Window) -> (u32, u32) {
        let mut tmp_win : c_ulong = 0;
        let mut x : c_int = 0;
        let mut y : c_int = 0;
        let mut tmp : c_int = 0;
        let mut tmp2 : c_uint = 0;
        unsafe {
            XQueryPointer(self.display, window as c_ulong, &mut tmp_win, &mut tmp_win,
                          &mut x, &mut y, &mut tmp, &mut tmp, &mut tmp2);
        }

        (x as u32, y as u32)
    }

    fn warp_pointer(&self, window: Window, x: u32, y: u32) {
        unsafe {
            XWarpPointer(self.display, 0, window, 0, 0, 0, 0, x as i32, y as i32);
        }
    }

    fn overrides_redirect(&self, window: Window) -> bool {
        unsafe {
            let mut attributes : XWindowAttributes = uninitialized();
            XGetWindowAttributes(self.display, window as c_ulong, &mut attributes);
            attributes.override_redirect != 0
        }
    }
}
