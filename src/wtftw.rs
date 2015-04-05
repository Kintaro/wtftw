#[macro_use]
extern crate log;
extern crate getopts;
extern crate rustc_serialize;
extern crate wtftw_core;
extern crate wtftw_xlib;

use std::env;
use getopts::Options;
use rustc_serialize::json;
use wtftw_core::config::Config;
use wtftw_core::window_manager::WindowManager;
use wtftw_core::window_system::*;
use wtftw_xlib::XlibWindowSystem;

pub fn parse_window_ids(ids: &str) -> Vec<(Window, u32)> {
    json::decode(ids).unwrap()
}

fn main() {
    // Parse command line arguments
    let args : Vec<String> = env::args().collect();

    let mut options = Options::new();
    options.optopt("r", "resume", "list of window IDs to capture in resume", "WINDOW");
    let matches = match options.parse(args.into_iter().skip(1).collect::<Vec<_>>()) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string())
    };

    // Create a default config.generaluration
    let mut config = Config::initialize();
    // Initialize window system. Use xlib here for now
    debug!("initialize window system");
    let window_system = XlibWindowSystem::new();
    // Create the actual window manager
    debug!("create window manager");
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
        window_system.grab_keys(vec!(command.clone()));
    }

    for (&command, _) in config.internal.mouse_handlers.iter() {
        window_system.grab_button(command);
    }

    let window_ids = if matches.opt_present("r") {
        debug!("trying to manage pre-existing windows");
        debug!("found {}", matches.opt_str("r").unwrap());
        parse_window_ids(&matches.opt_str("r").unwrap())
    } else {
        Vec::new()
    };

    for (window, workspace) in window_ids {
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
                    debug!("screen configuration changed. rescreen");
                    window_manager = window_manager.rescreen(&window_system);
                }
            },
            // A window asked to be reconfigured (i.e. resized, border change, etc.)
            WindowSystemEvent::ConfigurationRequest(window, window_changes, mask) => {
                let floating = window_manager.workspaces.floating.iter().any(|(&x, _)| x == window) ||
                    !window_manager.workspaces.contains(window);
                window_system.configure_window(window, window_changes, mask, floating);
                window_manager = window_manager.windows(&window_system, &config.general, |x| x.clone());
            },
            // A new window was created, so we need to manage
            // it unless it is already managed by us.
            WindowSystemEvent::WindowCreated(window) => {
                if window_manager.is_window_managed(window) || window_system.overrides_redirect(window) {
                    continue;
                }

                window_manager = window_manager.manage(&window_system, window, &config.general)
                                               .windows(&window_system, &config.general,
                                                        |x| (config.internal.manage_hook)(x.clone(),
                                                        &window_system, window));
            },
            WindowSystemEvent::WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager = if synthetic || !window_manager.is_waiting_unmap(window) {
                        window_manager.unmanage(&window_system, window, &config.general)
                    } else {
                        window_manager.update_unmap(window)
                    }
                }
            },
            WindowSystemEvent::WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager = window_manager.unmanage(&window_system, window, &config.general)
                        .remove_from_unmap(window);
                }
            },
            // The mouse pointer entered a window's region. If focus following
            // is enabled, we need to set focus to it.
            WindowSystemEvent::Enter(window) => {
                if config.general.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_manager = window_manager.focus(window, &window_system, &config.general);
                }
            },
            // Mouse button has been pressed. We need to check if there is a mouse handler
            // associated and if necessary, call it. Otherwise it results in a focus action.
            WindowSystemEvent::ButtonPressed(window, subwindow, button, _, _) => {
                let is_root = window_system.get_root() == window;
                let is_sub_root = window_system.get_root() == subwindow || subwindow == 0;
                let act = config.internal.mouse_handlers.get(&button);

                match act {
                    Some(ref action) => {
                        // If it's a root window, then it's an event we grabbed
                        if is_root && !is_sub_root {
                            let local_window_manager = window_manager.clone();
                            window_manager = action(local_window_manager, &window_system,
                                                          &config.general, subwindow);
                        }
                    }
                    None => {
                        // Otherwise just clock to focus
                        if !is_root {
                            window_manager = window_manager.focus(window, &window_system, &config.general);
                        }
                    }
                }
            },
            WindowSystemEvent::ButtonReleased => {
                // If we were dragging, release the pointer and
                // reset the dragging closure
                if let Some(_) = window_manager.dragging {
                    window_system.ungrab_pointer();
                    window_manager.dragging = None;
                }
            }
            WindowSystemEvent::KeyPressed(_, key) => {
                if config.internal.key_handlers.contains_key(&key) {
                    let local_window_manager = window_manager.clone();
                    window_manager = config.internal.key_handlers[&key](local_window_manager,
                        &window_system, &config.general);
                }
            },
            WindowSystemEvent::MouseMotion(x, y) => {
                let local_window_manager = window_manager.clone();
                if let Some(drag) = window_manager.dragging {
                    window_manager = drag(x, y, local_window_manager, &window_system);
                    window_system.remove_motion_events();
                }
            },
            _ => ()
        };

        if let Some(ref mut loghook) = config.internal.loghook {
            loghook(window_manager.clone(), &window_system);
        }
    }
}
