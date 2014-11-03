#![feature(phase)]
#[phase(plugin, link)] 
extern crate log;

use config::Config;
use core::Workspaces;
use layout::Layout;
use layout::TallLayout;
use layout::tile;
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

    info!("WTFTW - Window Tiling For The Win");
    info!("Starting wtftw on {} screen(s)", window_system.get_number_of_screens());

    for (i, &Rectangle(_, _, w, h)) in window_system.get_screen_infos().iter().enumerate() {
        info!("Display {}: {}x{}", i, w, h);
    }

    let mut workspaces = Workspaces::new(String::from_str("Tall"), config.tags, window_system.get_screen_infos());
    let layout = TallLayout { num_master: 1, increment_ratio: 0.03, ratio: 0.5 }; 

    loop {
        match window_system.get_event() {
            WindowCreated(window) => {
                let r = window_system.get_screen_infos()[0];
                window_system.show_window(window);
                workspaces.current.workspace.add(window);

                let screen = &workspaces.current;
                let workspace = &screen.workspace;
                let window_layout = layout.apply_layout(screen.screen_detail, &workspace.stack); 

                for &(win, Rectangle(x, y, w, h)) in window_layout.iter() {
                    window_system.resize_window(win, w - config.spacing, h - config.spacing);
                    window_system.move_window(win, x + config.spacing / 2, y + config.spacing / 2);
                    window_system.set_window_border_width(win, config.border_width);
                }

                debug!("Created window \"{}\"", window_system.get_window_name(window));
            },
            WindowDestroyed(window) => {
                
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
