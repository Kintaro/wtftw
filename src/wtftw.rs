#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use config::Config;
use core::Workspaces;
use layout::Layout;
use layout::TallLayout;
use layout::tile;
use window_manager::WindowManager;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_system::{
    Enter,
    Leave,
    WindowCreated,
    WindowDestroyed,
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
    // Create the actual window manager
    let mut window_manager = WindowManager::new(&window_system, &config); 

    // Output some initial information
    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_number_of_screens());

    for (i, &Rectangle(_, _, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        info!("Display {}: {}x{}", i, w, h);
    }

    // Enter the event loop and just listen for events
    loop {
        match window_system.get_event() {
            WindowCreated(window) => {
                if !window_manager.is_window_managed(window) {
                    window_manager.manage(&mut window_system, window, &config);
                }
            },
            WindowDestroyed(window) => {
                
            },
            Enter(window) => {
                if config.focus_follows_mouse {
                    window_system.set_window_border_color(window, config.focus_border_color);
                }
                debug!("Entered window \"{}\"", window_system.get_window_name(window));
            },
            Leave(window) => {
                if config.focus_follows_mouse {
                    window_system.set_window_border_color(window, config.border_color);
                }
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
