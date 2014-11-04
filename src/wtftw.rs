#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use config::Config;
use window_manager::WindowManager;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_system::{
    ConfigurationRequest,
    Enter,
    Leave,
    KeyPressed,
    WindowCreated,
    WindowDestroyed,
    WindowUnmapped,
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
    let config = Config::default();
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
            ConfigurationRequest(window) => {
            },
            WindowCreated(window) => {
                if !window_manager.is_window_managed(window) {
                    window_manager.manage(&mut window_system, window, &config);
                }
            },
            WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager.unmanage(&mut window_system, window, &config);
                }
            },
            WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager.unmanage(&mut window_system, window, &config); 
                }
            },
            Enter(window) => {
                if config.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_system.set_window_border_color(window, config.focus_border_color);
                }
            },
            Leave(window) => {
                if config.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_system.set_window_border_color(window, config.border_color);
                }
            },
            KeyPressed(_, key, mask) => {
                if mask & 8 != 0 && key >= 10 && key <= 18 && (key - 10) < config.tags.len() {
                    debug!("switching workspace");
                    window_manager.view(&mut window_system, key - 10, &config);
                }

                if mask & 9 == 9 && key == 36 {
                    let (terminal, args) = config.terminal.clone();
                    let arguments : Vec<String> = args.split(' ').map(String::from_str).collect();
                    spawn(proc() {
                        Command::new(terminal).args(arguments.as_slice()).detached().spawn();
                    });
                }

                if mask & 9 == 9 && key == 24 {
                    break;
                }
            }
            _ => ()
        }
    }
}
