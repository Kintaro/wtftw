extern crate serialize;
extern crate rustc;
extern crate rustc_trans;
extern crate syntax;
extern crate libc;

use std::os::homedir;
use std::collections::BTreeMap;
use core::Workspaces;
use window_system::*;
use window_manager::WindowManager;
use handlers::{ KeyHandler, MouseHandler, ManageHook, StartupHook, LogHook };
use handlers::default::{ exit, restart, start_terminal };
use layout::{ Layout, TallLayout };

use std::mem;
use std::io::{ USER_DIR, File };
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::{ Command, Process, ExitStatus };
use std::dynamic_lib::DynamicLibrary;
use std::rc::Rc;
use std::sync::RWLock;

pub struct GeneralConfig<'a> {
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
    pub mod_mask: KeyModifiers,
    pub pipes: Vec<Rc<RWLock<Process>>>,
    pub layout: Box<Layout + 'a>
}

impl<'a> Clone for GeneralConfig<'a> {
    fn clone(&self) -> GeneralConfig<'a> {
        GeneralConfig {
            focus_follows_mouse: self.focus_follows_mouse,
            focus_border_color: self.focus_border_color,
            border_color: self.border_color,
            border_width: self.border_width,
            spacing: self.spacing,
            terminal: self.terminal.clone(),
            logfile: self.logfile.clone(),
            tags: self.tags.clone(),
            launcher: self.launcher.clone(),
            mod_mask: self.mod_mask.clone(),
            pipes: self.pipes.clone(),
            layout: self.layout.copy()
        }
    }
}

pub struct InternalConfig<'a> {
    pub library: Option<DynamicLibrary>,
    pub key_handlers: BTreeMap<KeyCommand, KeyHandler<'a>>,
    pub mouse_handlers: BTreeMap<MouseCommand, MouseHandler<'a>>,
    pub manage_hook: ManageHook<'a>,
    pub startup_hook: StartupHook<'a>,
    pub loghook: Option<LogHook<'a>>,
}

/// Common configuration options for the window manager.
pub struct Config<'a> {
    pub general: GeneralConfig<'a>,
    pub internal: InternalConfig<'a>
}

impl<'a> Config<'a> {
    /// Create the Config from a json file
    pub fn initialize<'b>() -> Config<'b> {
        // Default version of the config, for fallback
        Config {
            general: GeneralConfig {
                focus_follows_mouse: true,
                focus_border_color:  0x00B6FFB0,
                border_color:        0x00444444,
                border_width:        2,
                spacing:             10,
                mod_mask:            MOD1MASK,
                terminal:            (String::from_str("xterm"), String::from_str("")),
                logfile:             format!("{}/.wtftw.log", homedir().unwrap().to_c_str()),
                tags:                vec!(
                    String::from_str("1: term"),
                    String::from_str("2: web"),
                    String::from_str("3: code"),
                    String::from_str("4: media")),
                launcher:            String::from_str("dmenu_run"),
                pipes:               Vec::new(),
                layout:              box TallLayout { num_master: 1, increment_ratio: 0.3/100.0, ratio: 0.5 }
            },
            internal: InternalConfig {
                library:      None,
                key_handlers: BTreeMap::new(),
                mouse_handlers: BTreeMap::new(),
                manage_hook:  box move |&: m: Workspaces<'b>, _: &WindowSystem, _: Window| -> Workspaces<'b> {
                    m.clone()
                },
                startup_hook: box move |&: m: WindowManager<'b>, _: &WindowSystem, _: &Config| -> WindowManager<'b> {
                    m.clone()
                },
                loghook:      None,
            }
        }
    }

    pub fn default_configuration(&mut self, w: &WindowSystem) {
        let mod_mask = self.general.mod_mask.clone();
        self.add_key_handler(w.get_keycode_from_string("Return"), mod_mask | SHIFTMASK,
            box |&: m, w, c| start_terminal(m, w, c));
        self.add_key_handler(w.get_keycode_from_string("q"), mod_mask,
            box |&: m, w, c| restart(m, w, c));
        self.add_key_handler(w.get_keycode_from_string("q"), mod_mask | SHIFTMASK,
            box |&: m, w, c| exit(m, w, c));
    }

    pub fn get_mod_mask(&self) -> KeyModifiers {
        self.general.mod_mask.clone()
    }

    pub fn add_key_handler(&mut self, key: u64, mask: KeyModifiers, keyhandler: KeyHandler<'a>) {
        self.internal.key_handlers.insert(KeyCommand::new(key, mask), keyhandler);
    }

    pub fn add_mouse_handler(&mut self, button: MouseButton, mask: KeyModifiers, 
                             mousehandler: MouseHandler<'a>) {
        self.internal.mouse_handlers.insert(MouseCommand::new(button, mask), mousehandler);
    }

    pub fn set_manage_hook(&mut self, hook: ManageHook<'a>) {
        self.internal.manage_hook = hook;
    }

    pub fn set_log_hook(&mut self, hook: LogHook<'a>) {
        self.internal.loghook = Some(hook);
    }

    pub fn compile_and_call(&mut self, m: &mut WindowManager, w: &WindowSystem) {
        let home = homedir().unwrap().to_c_str();
        let toml = format!("{}/.wtftw/Cargo.toml", home);

        if !Path::new(format!("{}/.wtftw", home)).exists() {
            fs::mkdir(&Path::new(format!("{}/.wtftw", home)), USER_DIR).unwrap();
        }

        if !Path::new(toml.clone()).exists() {
            let file = File::create(&Path::new(toml));
            file.unwrap().write_line("[project]\n\
                                     name = \"config\"\n\
                                     version = \"0.0.0\"\n\
                                     authors = [\"wtftw\"]\n\n\
                                     [dependencies.wtftw]\n\
                                     git = \"https://github.com/Kintaro/wtftw.git\"\n\n\
                                     [lib]\n\
                                     name = \"config\"\n\
                                     crate-type = [\"dylib\"]").unwrap();
        }

        let config_source = format!("{}/.wtftw/src/config.rs", home);
        if Path::new(config_source).exists() {
            self.compile();
            self.call(m, w)
        } else {
            self.default_configuration(w);
        }
    }

    pub fn compile(&self) {
        let home = homedir().unwrap().to_c_str();

        info!("compiling config module");
        let output = Command::new("cargo")
            .cwd(&Path::new(format!("{}/.wtftw", home)))
            .arg("build")//.arg("--release")
            .env("RUST_LOG", "none")
            .output().unwrap();

        if output.status == ExitStatus(0) {
            info!("config module compiled");
        } else {
            error!("error compiling config module");
            Command::new("xmessage").arg("\"error compiling config module\"").status().unwrap();
        }
    }

    pub fn call(&mut self, m: &mut WindowManager, w: &WindowSystem) {
        let home = homedir().unwrap().to_c_str();

        debug!("looking for config module");
        let contents = fs::readdir(&Path::new(format!("{}/.wtftw/target", home))).unwrap();
        let libname = contents.iter().find(|&x|
                             x.is_file() &&
                             x.filename_str().unwrap().contains("libconfig") &&
                             x.extension_str().unwrap().contains("so")).unwrap();

        if let Ok(lib) = DynamicLibrary::open(Some(libname)) {
            unsafe {
                if let Ok(symbol) = lib.symbol("configure") {
                    let result = mem::transmute::<*mut u8, extern fn(&mut WindowManager,
                                                        &WindowSystem,
                                                        &mut Config)>(symbol);

                    self.internal.library = Some(lib);
                    result(m, w, self);
                } else {
                    error!("Error loading config module")
                }
            }
        }
    }
}
