#![feature(unboxed_closures, unboxed_closure_sugar, overloaded_calls)]
#![feature(phase)]
#![feature(globs)]
#[phase(plugin, link)]
extern crate log;
extern crate serialize;

use std::ops::Fn;
use config::Config;
use logger::FileLogger;
use window_manager::WindowManager;
use window_system::*;
use xlib_window_system::XlibWindowSystem;
use handlers::default::*;

pub mod config;
pub mod core;
pub mod handlers;
pub mod layout;
pub mod logger;
pub mod window_manager;
pub mod window_system;
pub mod xlib_window_system;

include!("local_config.rs")

fn main() {
    // Initialize window system. Use xlib here for now
    let window_system = XlibWindowSystem::new();
    // Create a default configuration
    let mut config = Config::initialize();
    // Create the actual window manager
    let mut window_manager = WindowManager::new(&window_system, &config);
    //
    let logger = FileLogger::new(&config.logfile);
    log::set_logger(box logger);

    // Output some initial information
    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_screen_infos().len());

    for (i, &Rectangle(x, y, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        debug!("Display {}: {}x{} ({}, {})", i, w, h, x, y);
    }

    configure(&mut window_manager, &window_system, &mut config);

    for (command, _) in config.key_handlers.iter() {
        debug!("grabbing command {}", command);
        window_system.grab_keys(vec!(command.clone()));
    }

    // Enter the event loop and just listen for events
    loop {
        let event = window_system.get_event();
        match event {
            ClientMessageEvent(_) => {
            },
            // The X11/Wayland configuration changed, so we need to readjust the
            // screen configurations.
            ConfigurationNotification(window) => {
                if window_system.get_root() == window {
                    debug!("X configuration changed. Rescreen");

                    // Output some initial information
                    info!("WTFTW - Window Tiling For The Win");
                    info!("Starting wtftw on {} screen(s)", window_system.get_number_of_screens());

                    for (i, &Rectangle(x, y, w, h)) in window_system.get_screen_infos().iter().enumerate() {
                        debug!("Display {}: {}x{} ({}, {})", i, w, h, x, y);
                    }

                    window_manager.rescreen(&window_system);
                }
            },
            // A window asked to be reconfigured (i.e. resized, border change, etc.)
            ConfigurationRequest(window, window_changes, mask) => {
                window_system.configure_window(window, window_changes, mask);
            },
            // A new window was created, so we need to manage
            // it unless it is already managed by us.
            WindowCreated(window) => {
                if !window_manager.is_window_managed(window) {
                    window_manager.manage(&window_system, window, &config);
                }
            },
            WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager.unmanage(&window_system, window, &config);
                }
            },
            WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager.unmanage(&window_system, window, &config);
                }
            },
            // The mouse pointer entered a window's region. If focus following
            // is enabled, we need to set focus to it.
            Enter(window) => {
                if config.focus_follows_mouse && window_manager.is_window_managed(window) {
                    let color = config.focus_border_color;
                    window_manager.unfocus_windows(&window_system, &config);
                    window_system.set_window_border_color(window, color);
                    window_system.focus_window(window);
                }
            },
            // The mouse pointer left a window's reagion. If focus following is enabled,
            // we need to reset the border color
            Leave(window) => {
                if config.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_system.set_window_border_color(window, config.border_color);
                }
            },
            KeyPressed(_, key) => {
                for (command, handler) in config.key_handlers.iter() {
                    if command == &key {
                        let h = handler.clone();
                        let local_window_manager = window_manager.clone();
                        window_manager = (**h).call((local_window_manager, &window_system, &config));
                        continue;
                    }
                }

                if key.mask == MOD1MASK | SHIFTMASK && key.key == config.exit_key {
                    break;
                }
            },
            _ => ()
        }
    }
}
