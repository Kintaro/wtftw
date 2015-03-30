extern crate libc;
extern crate rustc_serialize;

use core::workspaces::Workspaces;
use window_manager::WindowManager;
use window_system::WindowSystem;
use window_system::Window;
use config::{ GeneralConfig, Config };

pub type KeyHandler = Box<Fn(&mut WindowManager, &WindowSystem, &GeneralConfig)>;
pub type MouseHandler = Box<Fn(&mut WindowManager, &WindowSystem, &GeneralConfig, Window)>;
pub type ManageHook = Box<Fn(Workspaces, &WindowSystem, Window) -> Workspaces>;
pub type StartupHook = Box<Fn(&mut WindowManager, &WindowSystem, &Config)>;
pub type LogHook = Box<FnMut(WindowManager, &WindowSystem)>;

extern {
    pub fn waitpid(fd: libc::pid_t, status: *mut libc::c_int, options: libc::c_int) -> libc::pid_t;
}

/// Some default handlers for easier config scripts
pub mod default {
    use std::env;
    use std::ptr::null;
    use std::process::Command;
    use std::thread::spawn;
    use handlers::rustc_serialize::json;
    use core::workspaces::Workspaces;
    use window_manager::WindowManager;
    use window_system::WindowSystem;
    use window_system::Window;
    use config::GeneralConfig;
    use handlers::libc::funcs::posix88::unistd::execvp;
    use std::ffi::CString;

    pub fn start_terminal(_: &mut WindowManager, _: &WindowSystem,
                          config: &GeneralConfig) {
        let (terminal, args) = config.terminal.clone();
        let arguments : Vec<String> = if args.is_empty() {
            Vec::new()
        } else {
            args.split(' ').map(String::from_str).collect()
        };

        spawn(move || {
            debug!("spawning terminal");
            let command = if arguments.is_empty() {
                Command::new(&terminal).spawn()
            } else {
                Command::new(&terminal).args(&arguments[..]).spawn()
            };

            if let Err(_) = command {
                panic!("unable to start terminal")
            }
        });
    }

    pub fn start_launcher(_: &mut WindowManager, _: &WindowSystem,
                          config: &GeneralConfig) {
        let launcher = config.launcher.clone();
        spawn(move || {
            debug!("spawning launcher");
            match Command::new(&launcher).spawn() {
                Ok(_) => (),
                _     => panic!("unable to start launcher")
            }
        });
    }

    pub fn switch_to_workspace(window_manager: &mut WindowManager, window_system: &WindowSystem,
                               config: &GeneralConfig, index: usize) {
        window_manager.view(window_system, index as u32, config)
    }

    pub fn move_window_to_workspace(window_manager: &mut WindowManager, window_system: &WindowSystem,
                                    config: &GeneralConfig, index: usize) {
        window_manager.move_window_to_workspace(window_system, config, index as u32)
    }

    /// Restart the window manager by calling execvp and replacing the current binary
    /// with the new one in memory.
    /// Pass a list of all windows to it via command line arguments
    /// so it may resume work as usual.
    pub fn restart(window_manager: &mut WindowManager, _: &WindowSystem, c: &GeneralConfig) {
        // Get absolute path to binary
        let filename = env::current_dir().unwrap().join(&env::current_exe().unwrap());
        // Collect all managed windows
        let window_ids : String = json::encode(&window_manager.workspaces.all_windows_with_workspaces()).unwrap();

        // Create arguments
        let resume = &"--resume";
        let windows = window_ids;
        let filename_c = CString::new(filename.into_os_string().into_string().unwrap().as_bytes()).unwrap();

        for ref p in c.pipes.iter() {
            match p.write().unwrap().wait() {
                _ => ()
            }
        }

        unsafe {
            let mut slice : &mut [*const i8; 4] = &mut [
                filename_c.as_ptr(),
                CString::new(resume.as_bytes()).unwrap().as_ptr(),
                CString::new(windows.as_bytes()).unwrap().as_ptr(),
                null()
            ];
            execvp(filename_c.as_ptr(), slice.as_mut_ptr());
        }
    }

    /// Stop the window manager
    pub fn exit(_: &mut WindowManager, _: &WindowSystem, _: &GeneralConfig) { }

    pub fn shift(index: u32, workspace: Workspaces, window: Window) -> Workspaces {
        workspace.shift_window(index, window)
    }
}
