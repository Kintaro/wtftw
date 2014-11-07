#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;
extern crate serialize;

use config::Config;
use logger::FileLogger;
use window_manager::WindowManager;
use window_system::Rectangle;
use window_system::WindowSystem;
use window_system::{
    ClientMessageEvent,
    ConfigurationNotification,
    ConfigurationRequest,
    Enter,
    Leave,
    KeyPressed,
    KeyCommand,
    WindowCreated,
    WindowDestroyed,
    WindowUnmapped,
    Mod1Mask,
    ShiftMask
};
use xlib_window_system::XlibWindowSystem;
use std::io::process::Command;

pub mod config;
pub mod core;
pub mod layout;
pub mod logger;
pub mod window_manager;
pub mod window_system;
pub mod xlib_window_system;

fn main() {
    // Initialize window system. Use xlib here for now
    let mut window_system = XlibWindowSystem::new();
    // Create a default configuration
    let mut config = Config::initialize();
    // Create the actual window manager
    let mut window_manager = WindowManager::new(&window_system, &config);
    // 
    let logger = FileLogger::new(&config.current().logfile); //Not hotswappable
    log::set_logger(box logger);

    // Output some initial information
    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_screen_infos().len());

    for (i, &Rectangle(x, y, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        debug!("Display {}: {}x{} ({}, {})", i, w, h, x, y);
    }

    // Temporary hack
    window_system.grab_keys(vec!(
            KeyCommand { key: String::from_str("Return"), mask: Mod1Mask | ShiftMask },
            KeyCommand { key: String::from_str("q"), mask: Mod1Mask | ShiftMask },
            KeyCommand { key: String::from_str("p"), mask: Mod1Mask | ShiftMask },
            KeyCommand { key: String::from_str("s"), mask: Mod1Mask | ShiftMask },
    ));

    // Enter the event loop and just listen for events
    loop {
        let curr_conf = config.current();
        match window_system.get_event() {
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

                    window_manager.rescreen(&mut window_system, &curr_conf);
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
                    window_manager.manage(&mut window_system, window, &curr_conf);
                }
            },
            WindowUnmapped(window, synthetic) => {
                if synthetic && window_manager.is_window_managed(window) {
                    window_manager.unmanage(&mut window_system, window, &curr_conf);
                }
            },
            WindowDestroyed(window) => {
                if window_manager.is_window_managed(window) {
                    window_manager.unmanage(&mut window_system, window, &curr_conf);
                }
            },
            // The mouse pointer entered a window's region. If focus following
            // is enabled, we need to set focus to it.
            Enter(window) => {
               if curr_conf.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_system.set_window_border_color(window, curr_conf.focus_border_color);
                    window_system.focus_window(window);
                }
            },
            // The mouse pointer 
            Leave(window) => {
                if curr_conf.focus_follows_mouse && window_manager.is_window_managed(window) {
                    window_system.set_window_border_color(window, curr_conf.border_color);
                }
            },
            KeyPressed(_, key) => {
//                if key.mask == Mod1Mask && key >= 10 && key <= 18 && (key - 10) < curr_conf.tags.len() as u32 {
//                    debug!("switching workspace to {}", curr_conf.tags[key as uint - 10].clone());
//                    window_manager.view(&mut window_system, key - 10, &curr_conf);
//                } else 
                if key.mask == Mod1Mask | ShiftMask && key.key == curr_conf.terminal_key {
                    let (terminal, args) = curr_conf.terminal.clone();
                    let arguments : Vec<String> = args.split(' ').map(String::from_str).collect();
                    spawn(proc() {
                        Command::new(terminal).args(arguments.as_slice()).detached().spawn();
                    });
                } else if key.mask == Mod1Mask | ShiftMask && key.key == curr_conf.launch_key {
                    let launcher = curr_conf.launcher.clone();
                    spawn(proc() {
                        Command::new(launcher).detached().spawn();
                    });
                } else if key.mask == Mod1Mask | ShiftMask && key.key == curr_conf.save_config_key {
                    config = Config::initialize();
                    debug!("configuration reloaded!");
                } else if key.mask == Mod1Mask | ShiftMask && key.key == curr_conf.exit_key {
                    break;
                } 
            },
            _ => ()
        }
    }
}
