use window_system::WindowSystem;
use window_system::{
    WindowCreated
};
use xlib_window_system::XlibWindowSystem;

mod core;
mod window_system;
mod xlib_window_system;

fn main() {
    unsafe {
        let mut window_system = XlibWindowSystem::new();

        println!("Starting wtftw on display with {}x{}", 
                 window_system.get_display_width(0),
                 window_system.get_display_height(0));
        
        loop {
            if !window_system.event_pending() {
                continue;
            }

            match window_system.get_event() {
                WindowCreated(window) => {
                    let w = window_system.get_display_width(0);
                    let h = window_system.get_display_height(0);
                    window_system.show_window(window);
                    window_system.resize_window(window, w / 2, h);
                    window_system.set_window_border_color(window, 0x00FF0000);
                    window_system.set_window_border_width(window, 2);
                },
                _ => ()
            }
        }
    }
}
