#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use config::Config;
use core::Workspaces;
use layout::Layout;
use layout::tile;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_system::{
    Enter,
    Leave,
    WindowCreated,
    ButtonPressed
};
use xlib_window_system::XlibWindowSystem;
use std::io::process::Command;

pub mod config;
pub mod core;
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

    let mut windows = Vec::new();

    let workspaces = Workspaces::new(String::from_str("Tall"), config.tags, window_system.get_screen_infos());

    loop {
        match window_system.get_event() {
            WindowCreated(window) => {
                let r = window_system.get_screen_infos()[0];
                window_system.show_window(window);
                windows.push(window);

                let layout = tile(0.5, r, 1, windows.len());

                for (&win, &Rectangle(x, y, w, h)) in windows.iter().zip(layout.iter()) {
                    window_system.resize_window(win, w, h);
                    window_system.move_window(win, x, y);
                }

                debug!("Created window \"{}\"", window_system.get_window_name(window));
            },
            Enter(window) => {
                window_system.set_window_border_color(window, config.focus_border_color);
                debug!("Entered window \"{}\"", window_system.get_window_name(window));
            },
            Leave(window) => {
                window_system.set_window_border_color(window, config.border_color);
                debug!("Left window \"{}\"", window_system.get_window_name(window));
            },
            ButtonPressed(_, _, _, _, _) => {
                debug!("Button pressed!");
                let name = config.terminal.clone();
                spawn(proc(){
                    Command::new(name).spawn();
                });
            }
            _ => ()
        }
    }
}
