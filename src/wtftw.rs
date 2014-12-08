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

    // Create a default config.generaluration
    let mut config = Config::initialize();
    log::set_logger(box FileLogger::new(&config.general.logfile, false));
    // Initialize window system. Use xlib here for now
    let window_system = XlibWindowSystem::new();
    // Create the actual window manager
    let mut window_manager = WindowManager::new(&window_system, &config.general);

    // If available, compile the config.general file at ~/.wtftw/config.general.rs
    // and call the config.generalure method
    config.compile_and_call(&mut window_manager, &window_system);
    window_manager = WindowManager::new(&window_system, &config.general);

    // Output some initial information
    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_screen_infos().len());

    // Output information about displays
    for (i, &Rectangle(x, y, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        debug!("Display {}: {}x{} ({}, {})", i, w, h, x, y);
    }

    debug!("Size of keyhandlers after config.generaluration: {}", config.internal.key_handlers.len());

    for (command, _) in config.internal.key_handlers.iter() {
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
        window_manager = window_manager.view(&window_system, workspace, &config.general)
            .manage(&window_system, window, &config.general);
    }

    // Enter the event loop and just listen for events
    while window_manager.running {
        let event = window_system.get_event();
        match event {
            WindowSystemEvent::ClientMessageEvent(_) => {
            },
            // The X11/Wayland configuration changed, so we need to readjust the
            // screen configurations.
            WindowSystemEvent::ConfigurationNotification(window) => {
                if window_system.get_root() == window {
                    debug!("screen config.generaluration changed. Rescreen");
                    window_manager = window_manager.rescreen(&window_system);
                }
            },
            // A window asked to be reconfig.generalured (i.e. resized, border change, etc.)
            WindowSystemEvent::ConfigurationRequest(window, window_changes, mask) => {
                window_system.configure_window(window, window_changes, mask);
                window_manager = window_manager.windows(&window_system, &config.general, |x| x.clone());
            },
            // A new window was created, so we need to manage
            // it unless it is already managed by us.
            WindowSystemEvent::WindowCreated(window) => {
                if window_manager.is_window_managed(window) {
                    continue;
                }

                window_manager = window_manager.manage(&window_system, window, &config.general);
                window_manager = window_manager.windows(&window_system, &config.general,
                                                        |x| config.internal.manage_hook.call((x.clone(),
                                                        &window_system, window)));
            },
            WindowSystemEvent::WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager.unmanage(&window_system, window, &config.general);
                    // TODO: remove from mapped stack and from waitingUnmap stack
                }
            },
            WindowSystemEvent::WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager = window_manager.unmanage(&window_system, window, &config.general);
                }
            },
            // The mouse pointer entered a window's region. If focus following
            // is enabled, we need to set focus to it.
            WindowSystemEvent::Enter(window) => {
                if config.general.focus_follows_mouse && window_manager.is_window_managed(window) {
                    debug!("enter event on {}", window_system.get_window_name(window));
                    window_manager = window_manager.focus(window, &window_system, &config.general);
                }
            },
            // The mouse pointer left a window's reagion. If focus following is enabled,
            // we need to reset the border color
            //Leave(window) => {
            //    if config.general.focus_follows_mouse && window_manager.is_window_managed(window) {
            //        window_system.set_window_border_color(window, config.general.border_color);
            //    }
            //},
            WindowSystemEvent::KeyPressed(_, key) => {
                for (command, ref handler) in config.internal.key_handlers.iter() {
                    if command == &key {
                        let local_window_manager = window_manager.clone();
                        debug!("calling handler");
                        window_manager = handler.call((local_window_manager, &window_system, &config.general));
                        continue;
                    }
                }
            },
            _ => ()
        };

        if let Some(ref mut loghook) = config.internal.loghook {
            loghook.call_mut((window_manager.clone(), &window_system));
        }
    }
}
