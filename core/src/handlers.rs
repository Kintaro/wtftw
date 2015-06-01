extern crate libc;
extern crate rustc_serialize;

use std::rc::Rc;
use core::workspaces::Workspaces;
use window_manager::WindowManager;
use window_system::WindowSystem;
use window_system::Window;
use config::{ GeneralConfig, Config };

pub type KeyHandler = Box<Fn(WindowManager, Rc<WindowSystem>, &GeneralConfig) -> WindowManager>;
pub type MouseHandler = Box<Fn(WindowManager, Rc<WindowSystem>, &GeneralConfig, Window) -> WindowManager>;
pub type ManageHook = Box<Fn(Workspaces, Rc<WindowSystem>, Window) -> Workspaces>;
pub type StartupHook = Box<Fn(WindowManager, Rc<WindowSystem>, &Config) -> WindowManager>;
pub type LogHook = Box<FnMut(WindowManager, Rc<WindowSystem>)>;

extern {
    pub fn waitpid(fd: libc::pid_t, status: *mut libc::c_int, options: libc::c_int) -> libc::pid_t;
}

/// Some default handlers for easier config scripts
pub mod default {
    use std::env;
    use std::ptr::null;
    use std::process::Command;
    use std::thread::spawn;
    use std::borrow::ToOwned;
    use handlers::rustc_serialize::json;
    use core::workspaces::Workspaces;
    use window_manager::WindowManager;
    use window_system::WindowSystem;
    use window_system::Window;
    use config::GeneralConfig;
    use handlers::libc::funcs::posix88::unistd::execvp;
    use std::ffi::CString;
    use std::rc::Rc;
    use std::ops::Deref;

    pub fn start_terminal(window_manager: WindowManager, _: Rc<WindowSystem>,
                          config: &GeneralConfig) -> WindowManager {
        let (terminal, args) = config.terminal.clone();
        let arguments : Vec<String> = if args.is_empty() {
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

            if let Err(_) = command {
                panic!("unable to start terminal")
            }
        });

        window_manager.clone()
    }

    pub fn start_launcher(window_manager: WindowManager, _: Rc<WindowSystem>,
                          config: &GeneralConfig) -> WindowManager {
        let launcher = config.launcher.clone();
        spawn(move || {
            debug!("spawning launcher");
            match Command::new(&launcher).spawn() {
                Ok(_) => (),
                _     => panic!("unable to start launcher")
            }
        });

        window_manager.clone()
    }

    pub fn switch_to_workspace(window_manager: WindowManager, window_system: Rc<WindowSystem>,
                               config: &GeneralConfig, index: usize) -> WindowManager {
        window_manager.view(window_system.deref(), index as u32, config)
    }

    pub fn move_window_to_workspace(window_manager: WindowManager, window_system: Rc<WindowSystem>,
                                    config: &GeneralConfig, index: usize) -> WindowManager {
        window_manager.move_window_to_workspace(window_system.deref(), config, index as u32)
    }

    /// Restart the window manager by calling execvp and replacing the current binary
    /// with the new one in memory.
    /// Pass a list of all windows to it via command line arguments
    /// so it may resume work as usual.
    pub fn restart<'a>(window_manager: WindowManager, _: Rc<WindowSystem>, c: &GeneralConfig) -> WindowManager {
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

        window_manager.clone()
    }

    /// Stop the window manager
    pub fn exit(w: WindowManager, _: Rc<WindowSystem>, _: &GeneralConfig) -> WindowManager {
        WindowManager { running: false, dragging: None, workspaces: w.workspaces, waiting_unmap: w.waiting_unmap.clone() }
    }

    pub fn shift(index: u32, workspace: Workspaces, window: Window) -> Workspaces {
        workspace.shift_window(index, window)
    }
}
