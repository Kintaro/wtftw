#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use window_system::WindowSystem;
use window_system::{
    Enter,
    Leave,
    WindowCreated
};
use xlib_window_system::XlibWindowSystem;

mod core;
mod window_system;
mod xlib_window_system;

fn main() {
    // Initialize window system. Use xlib here for now
    let mut window_system = XlibWindowSystem::new();

    info!("Starting wtftw on display with {}x{}", 
             window_system.get_display_width(0),
             window_system.get_display_height(0));

    let mut x = 0;

    loop {
        match window_system.get_event() {
            WindowCreated(window) => {
                let w = window_system.get_display_width(0);
                let h = window_system.get_display_height(0);
                window_system.show_window(window);
                window_system.resize_window(window, w / 2, h);
                window_system.move_window(window, x, 0);
                window_system.set_window_border_color(window, 0x00FF00FF);
                window_system.set_window_border_width(window, 2);
                x += w / 2;

                debug!("Created window \"{}\"", window_system.get_window_name(window));
            },
            Enter(window) => {
                window_system.set_window_border_color(window, 0x00FF0000);
                debug!("Entered window \"{}\"", window_system.get_window_name(window));
            },
            Leave(window) => {
                window_system.set_window_border_color(window, 0x00FFFFFF);
                debug!("Left window \"{}\"", window_system.get_window_name(window));
            }
            _ => ()
        }
    }
}
