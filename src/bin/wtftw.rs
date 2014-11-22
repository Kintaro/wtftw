#![feature(globs)]
#![feature(phase)]
#[phase(plugin, link)]
extern crate log;
extern crate getopts;
extern crate serialize;
extern crate wtftw;

use std::ops::Fn;
use std::os;
use getopts::{ optopt, getopts };
use serialize::json;
use wtftw::config::Config;
use wtftw::logger::FileLogger;
use wtftw::window_manager::WindowManager;
use wtftw::window_system::*;
use wtftw::xlib_window_system::XlibWindowSystem;
use wtftw::configure;

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

    // Initialize window system. Use xlib here for now
    let window_system = XlibWindowSystem::new();
    // Create a default configuration
    let mut config = Config::initialize(&window_system);
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
                        window_manager = handler.call((local_window_manager, &window_system, &config));
                        continue;
                    }
                }
            },
            _ => ()
        }
    }
}
