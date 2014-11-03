#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use config::Config;
use layout::Layout;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_system::{
    Enter,
    Leave,
    WindowCreated
};
use xlib_window_system::XlibWindowSystem;

pub mod config;
pub mod layout;
pub mod window_manager;
pub mod window_system;
pub mod xlib_window_system;

fn main() {
    // Initialize window system. Use xlib here for now
    let mut window_system = XlibWindowSystem::new();
    // Create a default configuration
    let mut config = Config::default();

    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_number_of_screens());

    for (i, &Rectangle(_, _, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        info!("Display {}: {}x{}", i, w, h);
    }

    let mut x = 0;

    loop {
        match window_system.get_event() {
            WindowCreated(window) => {
                let w = window_system.get_display_width(0);
                let h = window_system.get_display_height(0);
                window_system.show_window(window);
                window_system.resize_window(window, w / 2, h);
                window_system.move_window(window, x, 0);
                window_system.set_window_border_color(window, config.border_color);
                window_system.set_window_border_width(window, config.border_width);
                x += w / 2;

                debug!("Created window \"{}\" at {}", window_system.get_window_name(window), x);
            },
            Enter(window) => {
                window_system.set_window_border_color(window, config.focus_border_color);
                debug!("Entered window \"{}\"", window_system.get_window_name(window));
            },
            Leave(window) => {
                window_system.set_window_border_color(window, config.border_color);
                debug!("Left window \"{}\"", window_system.get_window_name(window));
            }
            _ => ()
        }
    }
}
