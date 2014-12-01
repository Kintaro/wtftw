#![feature(globs)]
#![feature(phase)]
#[phase(plugin, link)]
extern crate log;
extern crate getopts;
extern crate serialize;
extern crate wtftw_core;
extern crate wtftw_xlib;

use std::ops::Fn;
use std::os;
use getopts::{ optopt, getopts };
use serialize::json;
use wtftw_core::config::Config;
use wtftw_core::logger::FileLogger;
use wtftw_core::window_manager::WindowManager;
use wtftw_core::window_system::*;
use wtftw_xlib::XlibWindowSystem;

pub fn parse_window_ids(ids: &str) -> Vec<(Window, u32)> {
    json::decode(ids).unwrap()
}

fn main() {
    // Parse command line arguments
    let args : Vec<String> = os::args();

    let opts = [
        optopt("r", "resume", "list of window IDs to capture in resume", "WINDOW")
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string())
    };

    // Create a default configuration
    let mut config = Config::initialize();
    log::set_logger(box FileLogger::new(&config.logfile, false));
    // Initialize window system. Use xlib here for now
    let window_system = XlibWindowSystem::new();
    // Create the actual window manager
    let mut window_manager = WindowManager::new(&window_system, &config);

    // If available, compile the config file at ~/.wtftw/config.rs
    // and call the configure method
    let old = log::set_logger(box FileLogger::new(&config.logfile, true));
    config.compile_and_call(&mut window_manager, &window_system);
    config.call(&mut window_manager, &window_system);
    log::set_logger(old.unwrap());

    // Output some initial information
    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_screen_infos().len());

    // Output information about displays
    for (i, &Rectangle(x, y, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        debug!("Display {}: {}x{} ({}, {})", i, w, h, x, y);
    }

    debug!("Size of keyhandlers after configuration: {}", config.key_handlers.len());

    for (command, _) in config.key_handlers.iter() {
        debug!("grabbing command {}", command);
        window_system.grab_keys(vec!(command.clone()));
    }

    let window_ids = if matches.opt_present("r") {
        debug!("found {}", matches.opt_str("r").unwrap());
        parse_window_ids(matches.opt_str("r").unwrap().as_slice())
    } else {
        Vec::new()
    };

    for &(window, workspace) in window_ids.iter() {
        debug!("re-inserting window {}", window);
        window_manager = window_manager.view(&window_system, workspace, &config)
            .manage(&window_system, window, &config);
    }

    // Enter the event loop and just listen for events
    while window_manager.running {
        let event = window_system.get_event();
        match event {
            ClientMessageEvent(_) => {
            },
            // The X11/Wayland configuration changed, so we need to readjust the
            // screen configurations.
            ConfigurationNotification(window) => {
                if window_system.get_root() == window {
                    debug!("screen configuration changed. Rescreen");
                    window_manager = window_manager.rescreen(&window_system);
                }
            },
            // A window asked to be reconfigured (i.e. resized, border change, etc.)
            ConfigurationRequest(window, window_changes, mask) => {
                window_system.configure_window(window, window_changes, mask);
                window_manager = window_manager.windows(&window_system, &config, |x| x.clone());
            },
            // A new window was created, so we need to manage
            // it unless it is already managed by us.
            WindowCreated(window) => {
                if !window_manager.is_window_managed(window) {
                    window_manager = window_manager.manage(&window_system, window, &config);
                    window_manager = window_manager.windows(&window_system, &config,
                                                            |x| config.manage_hook.call((x.clone(),
                                                                         &window_system, window)));
                }
            },
            WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager.unmanage(&window_system, window, &config);
                    // TODO: remove from mapped stack and from waitingUnmap stack
                }
            },
            WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager = window_manager.unmanage(&window_system, window, &config);
                }
            },
            // The mouse pointer entered a window's region. If focus following
            // is enabled, we need to set focus to it.
            Enter(window) => {
                if config.focus_follows_mouse && window_manager.is_window_managed(window) {
                    debug!("enter event on {}", window_system.get_window_name(window));
                    window_manager = window_manager.focus(window, &window_system, &config);
                }
            },
            // The mouse pointer left a window's reagion. If focus following is enabled,
            // we need to reset the border color
            //Leave(window) => {
            //    if config.focus_follows_mouse && window_manager.is_window_managed(window) {
            //        window_system.set_window_border_color(window, config.border_color);
            //    }
            //},
            KeyPressed(_, key) => {
                for (command, ref handler) in config.key_handlers.iter() {
                    if command == &key {
                        let local_window_manager = window_manager.clone();
                        debug!("calling handler");
                        window_manager = handler.call((local_window_manager, &window_system, &config));
                        continue;
                    }
                }
            },
            _ => ()
        }
    }
}
