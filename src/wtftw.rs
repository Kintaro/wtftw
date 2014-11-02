extern crate libc;
extern crate xlib;

use libc::c_int;

use xlib::{
    XOpenDisplay,
    XDisplayWidth,
    XDisplayHeight,
    XDefaultRootWindow,
    XQueryTree,
    XNextEvent,
    XEvent,
    XAnyEvent,
    XGenericEvent,
    XMapRequestEvent,
    XPending,
    XSelectInput,
    XMoveWindow,
    XMapWindow,
    Window,
    Display
};

mod core;

fn main() {
    unsafe {
        let display = XOpenDisplay(std::ptr::null_mut());
        let root = XDefaultRootWindow(display);
        XSelectInput(display, root, 0x1E0000i64);

        println!("Starting wtftw on display with {}x{}", 
                 XDisplayWidth(display, 0),
                 XDisplayHeight(display, 0));

        
        let mut event = std::mem::uninitialized();;
        loop {
            if XPending(display) == 0 {
                continue;
            }

            XNextEvent(display, event);
            let event_ptr : *const c_int = std::mem::transmute(event);
            let event_type : c_int = *event_ptr;

            if event_type == 19 {
                let ev_ptr : *const XMapRequestEvent = std::mem::transmute(event);
                let ev = *ev_ptr;
                let window = ev.window;
                XMapWindow(display, window);
                XMoveWindow(display, window, 50, 50);
            }
            println!("{}", event_type);
        }
    }
}
