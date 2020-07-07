extern crate libc;
extern crate serde_json;

use crate::config::{Config, GeneralConfig};
use crate::core::workspaces::Workspaces;
use std::rc::Rc;
use crate::window_manager::WindowManager;
use crate::window_system::Window;
use crate::window_system::WindowSystem;

pub type KeyHandler = Box<dyn Fn(WindowManager, Rc<dyn WindowSystem>, &GeneralConfig) -> WindowManager>;
pub type MouseHandler =
    Box<dyn Fn(WindowManager, Rc<dyn WindowSystem>, &GeneralConfig, Window) -> WindowManager>;
pub type ManageHook = Box<dyn Fn(Workspaces, Rc<dyn WindowSystem>, Window) -> Workspaces>;
pub type StartupHook = Box<dyn Fn(WindowManager, Rc<dyn WindowSystem>, &Config) -> WindowManager>;
pub type LogHook = Box<dyn FnMut(WindowManager, Rc<dyn WindowSystem>)>;

extern "C" {
    pub fn waitpid(fd: libc::pid_t, status: *mut libc::c_int, options: libc::c_int) -> libc::pid_t;
}

/// Some default handlers for easier config scripts
pub mod default {
    use crate::config::GeneralConfig;
    use crate::core::workspaces::Workspaces;
    use crate::handlers::libc::execvp;
    use std::borrow::ToOwned;
    use std::env;
    use std::ffi::CString;
    use std::ops::Deref;
    use std::process::Command;
    use std::ptr::null;
    use std::rc::Rc;
    use std::thread::spawn;
    use crate::window_manager::WindowManager;
    use crate::window_system::Window;
    use crate::window_system::WindowSystem;

    pub fn start_terminal(
        window_manager: WindowManager,
        _: Rc<dyn WindowSystem>,
        config: &GeneralConfig,
    ) -> WindowManager {
        let (terminal, args) = config.terminal.clone();
        let arguments: Vec<String> = if args.is_empty() {
            Vec::new()
        } else {
            args.split(' ').map(|x| x.to_owned()).collect()
        };

        spawn(move || {
            debug!("spawning terminal");
            let command = if arguments.is_empty() {
                Command::new(&terminal).spawn()
            } else {
                Command::new(&terminal).args(&arguments[..]).spawn()
            };

            if command.is_err() {
                panic!("unable to start terminal")
            }
        });

        window_manager
    }

    pub fn start_launcher(
        window_manager: WindowManager,
        _: Rc<dyn WindowSystem>,
        config: &GeneralConfig,
    ) -> WindowManager {
        let launcher = config.launcher.clone();
        spawn(move || {
            debug!("spawning launcher");
            match Command::new(&launcher).spawn() {
                Ok(_) => (),
                _ => panic!("unable to start launcher"),
            }
        });

        window_manager
    }

    pub fn switch_to_workspace(
        window_manager: WindowManager,
        window_system: Rc<dyn WindowSystem>,
        config: &GeneralConfig,
        index: usize,
    ) -> WindowManager {
        window_manager.view(window_system.deref(), index as u32, config)
    }

    pub fn move_window_to_workspace(
        window_manager: WindowManager,
        window_system: Rc<dyn WindowSystem>,
        config: &GeneralConfig,
        index: usize,
    ) -> WindowManager {
        window_manager.move_window_to_workspace(window_system.deref(), config, index as u32)
    }

    /// Restart the window manager by calling execvp and replacing the current binary
    /// with the new one in memory.
    /// Pass a list of all windows to it via command line arguments
    /// so it may resume work as usual.
    pub fn restart(
        window_manager: WindowManager,
        _: Rc<dyn WindowSystem>,
        c: &GeneralConfig,
    ) -> WindowManager {
        // Get absolute path to binary
        let filename = env::current_dir()
            .unwrap()
            .join(&env::current_exe().unwrap());
        // Collect all managed windows
        let window_ids: String =
            json!(&window_manager.workspaces.all_windows_with_workspaces()).to_string();

        // Create arguments
        let resume = &"--resume";
        let windows = window_ids;
        let filename_c =
            CString::new(filename.into_os_string().into_string().unwrap().as_bytes()).unwrap();

        for p in c.pipes.iter() {
            p.write().unwrap().wait().unwrap();
        }

        let resume_str = CString::new(resume.as_bytes()).unwrap();
        let windows_str = CString::new(windows.as_bytes()).unwrap();

        unsafe {
            let slice: &mut [*const i8; 4] = &mut [
                filename_c.as_ptr(),
                resume_str.as_ptr(),
                windows_str.as_ptr(),
                null(),
            ];
            execvp(filename_c.as_ptr(), slice.as_mut_ptr());
        }

        window_manager
    }

    /// Stop the window manager
    pub fn exit(w: WindowManager, _: Rc<dyn WindowSystem>, _: &GeneralConfig) -> WindowManager {
        WindowManager {
            running: false,
            dragging: None,
            workspaces: w.workspaces,
            waiting_unmap: w.waiting_unmap,
        }
    }

    pub fn shift(index: u32, workspace: Workspaces, window: Window) -> Workspaces {
        workspace.shift_window(index, window)
    }
}
