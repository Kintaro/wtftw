extern crate rustc_serialize;
extern crate dylib;

use std::env;
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use core::workspaces::Workspaces;
use window_system::*;
use window_manager::WindowManager;
use handlers::{ KeyHandler, MouseHandler, ManageHook, StartupHook, LogHook };
use handlers::default::{ exit, restart, start_terminal };
use layout::{ Layout, TallLayout };

use std::mem;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::fs::PathExt;
use std::fs::{ read_dir, create_dir_all };
use std::process::Command;
use std::process::Child;
use self::dylib::DynamicLibrary;
use std::rc::Rc;
use std::sync::RwLock;
use std::thread::spawn;
use std::path::PathBuf;
use std::path::Path;

pub struct GeneralConfig {
    /// Whether focus follows mouse movements or
    /// only click events and keyboard movements.
    pub focus_follows_mouse: bool,
    /// Border color for focused windows.
    pub focus_border_color: u32,
    /// Border color for unfocused windows.
    pub border_color: u32,
    /// Border width. This is the same for both, focused and unfocused.
    pub border_width: u32,
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
    pub pipes: Vec<Rc<RwLock<Child>>>,
    pub layout: Box<Layout>
}

impl Clone for GeneralConfig {
    fn clone(&self) -> GeneralConfig {
        GeneralConfig {
            focus_follows_mouse: self.focus_follows_mouse,
            focus_border_color: self.focus_border_color,
            border_color: self.border_color,
            border_width: self.border_width,
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

pub struct InternalConfig {
    pub library: Option<DynamicLibrary>,
    pub key_handlers: BTreeMap<KeyCommand, KeyHandler>,
    pub mouse_handlers: BTreeMap<MouseCommand, MouseHandler>,
    pub manage_hook: ManageHook,
    pub startup_hook: StartupHook,
    pub loghook: Option<LogHook>,
    pub wtftw_dir: String,
}

impl InternalConfig {
    pub fn new(manage_hook: ManageHook, startup_hook: StartupHook, home: String) -> InternalConfig {
        InternalConfig {
            library: None,
            key_handlers: BTreeMap::new(),
            mouse_handlers: BTreeMap::new(),
            manage_hook: manage_hook,
            startup_hook: startup_hook,
            loghook: None,
            wtftw_dir: format!("{}/.wtftw", home)
        }
    }
}

/// Common configuration options for the window manager.
pub struct Config {
    pub general: GeneralConfig,
    pub internal: InternalConfig
}

impl Config {
    /// Create the Config from a json file
    pub fn initialize() -> Config {
        let home = env::home_dir().unwrap_or(PathBuf::from("./")).into_os_string().into_string().unwrap();
        // Default version of the config, for fallback
        let general_config = 
            GeneralConfig {
                focus_follows_mouse: true,
                focus_border_color:  0x00B6FFB0,
                border_color:        0x00444444,
                border_width:        2,
                mod_mask:            MOD1MASK,
                terminal:            ("xterm".to_owned(), "".to_owned()),
                logfile:             format!("{}/.wtftw.log", home),
                tags:                vec!(
                    "1: term".to_owned(),
                    "2: web".to_owned(),
                    "3: code".to_owned(),
                    "4: media".to_owned()),
                launcher:            "dmenu_run".to_owned(),
                pipes:               Vec::new(),
                layout:              Box::new(TallLayout { num_master: 1, increment_ratio: 0.3/100.0, ratio: 0.5 }),
            };
        
        let internal_config = InternalConfig::new(
            Box::new(Config::default_manage_hook),
            Box::new(Config::default_startup_hook),
            home); 

        Config {
            general: general_config,
            internal: internal_config
        }
    }

    pub fn default_manage_hook(m: Workspaces, _: &WindowSystem, _: Window) -> Workspaces {
        m
    }

    pub fn default_startup_hook(m: WindowManager, _: &WindowSystem, _: &Config) -> WindowManager {
        m
    }

    pub fn default_configuration(&mut self, w: &WindowSystem) {
        let mod_mask = self.general.mod_mask.clone();
        self.add_key_handler(w.get_keycode_from_string("Return"), mod_mask | SHIFTMASK,
            Box::new(|m, w, c| start_terminal(m, w, c)));
        self.add_key_handler(w.get_keycode_from_string("q"), mod_mask,
            Box::new(|m, w, c| restart(m, w, c)));
        self.add_key_handler(w.get_keycode_from_string("q"), mod_mask | SHIFTMASK,
            Box::new(|m, w, c| exit(m, w, c)));
    }

    pub fn get_mod_mask(&self) -> KeyModifiers {
        self.general.mod_mask.clone()
    }

    pub fn add_key_handler(&mut self, key: u64, mask: KeyModifiers, keyhandler: KeyHandler) {
        self.internal.key_handlers.insert(KeyCommand::new(key, mask), keyhandler);
    }

    pub fn add_mouse_handler(&mut self, button: MouseButton, mask: KeyModifiers,
                             mousehandler: MouseHandler) {
        self.internal.mouse_handlers.insert(MouseCommand::new(button, mask), mousehandler);
    }

    pub fn set_manage_hook(&mut self, hook: ManageHook) {
        self.internal.manage_hook = hook;
    }

    pub fn set_log_hook(&mut self, hook: LogHook) {
        self.internal.loghook = Some(hook);
    }

    pub fn compile_and_call(&mut self, m: &mut WindowManager, w: &WindowSystem) {
        let toml = format!("{}/Cargo.toml", self.internal.wtftw_dir.clone());

        if !Path::new(&self.internal.wtftw_dir.clone()).exists() {
            match create_dir_all(Path::new(&self.internal.wtftw_dir.clone())) {
                Ok(()) => (),
                Err(e) => panic!(format!("mkdir: {} failed with error {}", self.internal.wtftw_dir.clone(), e))
            }
        }

        if !Path::new(&toml.clone()).exists() {
            let file = File::create(Path::new(&toml).as_os_str());
            file.unwrap().write("[project]\n\
                                     name = \"config\"\n\
                                     version = \"0.0.0\"\n\
                                     authors = [\"wtftw\"]\n\n\
                                     [dependencies.wtftw]\n\
                                     git = \"https://github.com/Kintaro/wtftw.git\"\n\n\
                                     [lib]\n\
                                     name = \"config\"\n\
                                     crate-type = [\"dylib\"]".as_bytes()).unwrap();
        }

        let config_source = format!("{}/src/config.rs", self.internal.wtftw_dir.clone());
        if Path::new(&config_source).exists() && self.compile() {
            self.call(m, w)
        } else {
            self.default_configuration(w);
        }
    }

    pub fn compile(&self) -> bool {
        info!("updating dependencies");
        Command::new("cargo")
            .current_dir(&Path::new(&self.internal.wtftw_dir.clone()))
            .arg("update")
            .env("RUST_LOG", "none")
            .output().unwrap();
        info!("compiling config module");
        let output = Command::new("cargo")
            .current_dir(&Path::new(&self.internal.wtftw_dir.clone()))
            .arg("build")//.arg("--release")
            .env("RUST_LOG", "none")
            .output();

        match output {
            Ok(o) => {
                if o.status.success() {
                    info!("config module compiled");
                    true
                } else {
                    error!("error compiling config module");

                    spawn(move || {
                        Command::new("xmessage").arg("\"error compiling config module. run 'cargo build' in ~/.wtftw to get more info.\"").spawn().unwrap();
                    });
                    false
                }
            },
            Err(err) => {
                error!("error compiling config module");
                spawn(move || {
                    Command::new("xmessage").arg(err.description()).spawn().unwrap();
                });
                false
            }
        }
    }

    pub fn call(&mut self, m: &mut WindowManager, w: &WindowSystem) {
        debug!("looking for config module");
        let mut contents = read_dir(&Path::new(&format!("{}/target/debug", self.internal.wtftw_dir.clone()))).unwrap();
        let libname = contents.find(|x| {
                            match x {
                                &Ok(ref y) => y.path().into_os_string().as_os_str().to_str().unwrap().contains("libconfig"),
                                &Err(_) => false
                            }
        });

        if let Ok(lib) = DynamicLibrary::open(Some(&Path::new(&libname.unwrap().unwrap().path().as_os_str().to_str().unwrap()))) {
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
