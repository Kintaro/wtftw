extern crate serialize;

use std::os::homedir;
use std::collections::TreeMap;
use core::Workspaces;
use window_system::*;
use window_manager::WindowManager;
use handlers::KeyHandler;
use handlers::ManageHook;
use handlers::StartupHook;
use handlers::LogHook;

/// Common configuration options for the window manager.
pub struct Config<'a> {
    /// Whether focus follows mouse movements or
    /// only click events and keyboard movements.
    pub focus_follows_mouse: bool,
    /// Border color for focused windows.
    pub focus_border_color: u32,
    /// Border color for unfocused windows.
    pub border_color: u32,
    /// Border width. This is the same for both, focused and unfocused.
    pub border_width: u32,
    /// Default spacing between windows
    pub spacing: u32,
    /// Default terminal to start
    pub terminal: (String, String),
    /// Keybind for the terminal
    /// Path to the logfile
    pub logfile: String,
    /// Default tags for workspaces
    pub tags: Vec<String>,
    /// Default launcher application
    pub launcher: String,
    /// Keybind for the launcher and configuration reloading
    pub save_config_key: u64,
    pub exit_key: u64,
    pub key_handlers: TreeMap<KeyCommand, KeyHandler<'a>>,
    pub mod_mask: KeyModifiers,
    pub manage_hook: ManageHook<'a>,
    pub startup_hook: StartupHook<'a>,
    pub loghook: Option<LogHook>
}

impl<'a> Config<'a> {
    /// Create the Config from a json file
    pub fn initialize(window_system: &WindowSystem) -> Config<'a> {
        // Default version of the config, for fallback
        Config {
            focus_follows_mouse: true,
            focus_border_color:  0x00B6FFB0,
            border_color:        0x00444444,
            border_width:        2,
            spacing:             10,
            terminal:            (String::from_str("xterm"), String::from_str("-fg White -bg Black")),
            logfile:             format!("{}/.wtftw.log", homedir().unwrap().to_c_str()),
            tags:                vec!(
                String::from_str("1: term"),
                String::from_str("2: web"),
                String::from_str("3: code"),
                String::from_str("4: media")),
            launcher:            String::from_str("dmenu_run"),
            save_config_key:     window_system.get_keycode_from_string("s"),
            exit_key:            window_system.get_keycode_from_string("q"),
            key_handlers:        TreeMap::new(),
            mod_mask:            MOD1MASK,
            manage_hook:         box move |&: m: Workspaces, _: &WindowSystem, _: Window| -> Workspaces { m.clone() },
            startup_hook:        box move |&: m: WindowManager, _: &WindowSystem, _: &Config| -> WindowManager {
                m.clone()
            },
            loghook:             None
        }
    }

    pub fn get_mod_mask(&self) -> KeyModifiers {
        self.mod_mask
    }

    pub fn add_key_handler(&mut self, key: u64, mask: KeyModifiers, keyhandler: KeyHandler<'a>) {
        self.key_handlers.insert(KeyCommand::new(key, mask), keyhandler);
    }

    pub fn set_manage_hook(&mut self, hook: ManageHook<'a>) {
        self.manage_hook = hook;
    }
}
