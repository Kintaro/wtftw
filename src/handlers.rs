extern crate libc;

use core::Workspaces;
use window_manager::WindowManager;
use window_system::WindowSystem;
use window_system::Window;
use config::Config;

#[deriving(Clone)]
pub type KeyHandler<'a> = Box<Fn<(WindowManager, &'a WindowSystem + 'a, &'a Config<'a>), WindowManager> + 'static>;
pub type ManageHook = Box<Fn<(Workspaces, Window), Workspaces> + 'static>;

/// Some default handlers for easier config scripts
pub mod default {
    use std::mem;
    use std::os;
    use std::ptr::null;
    use std::io::process::Command;
    use serialize::json;
    use window_manager::WindowManager;
    use window_system::WindowSystem;
    use config::Config;
    use handlers::libc::funcs::posix88::unistd::execvp;

    pub fn start_terminal(window_manager: WindowManager, _: &WindowSystem,
                          config: &Config) -> WindowManager {
        let (terminal, args) = config.terminal.clone();
        let arguments : Vec<String> = args.split(' ').map(String::from_str).collect();
        spawn(proc() {
            debug!("spawning terminal");
            match Command::new(terminal).args(arguments.as_slice()).detached().spawn() {
                Ok(_) => (),
                _     => panic!("unable to start terminal")
            }
        });

        window_manager.clone()
    }

    pub fn start_launcher(window_manager: WindowManager, _: &WindowSystem,
                          config: &Config) -> WindowManager {
        let launcher = config.launcher.clone();
        spawn(proc() {
            debug!("spawning launcher");
            match Command::new(launcher).detached().spawn() {
                Ok(_) => (),
                _     => panic!("unable to start launcher")
            }
        });

        window_manager.clone()
    }

    pub fn switch_to_workspace(window_manager: WindowManager, window_system: &WindowSystem,
                               config: &Config, index: uint) -> WindowManager {
        window_manager.view(window_system, index as u32, config)
    }

    pub fn move_window_to_workspace(window_manager: WindowManager, window_system: &WindowSystem,
                                    config: &Config, index: uint) -> WindowManager {
        window_manager.move_window_to_workspace(window_system, config, index as u32)
    }

    pub fn restart(window_manager: WindowManager, _: &WindowSystem, _: &Config) -> WindowManager {
        let filename = os::make_absolute(&Path::new(os::args()[0].clone())).to_c_str();
        let window_ids : String = json::encode(&window_manager.workspaces.all_windows());

        let program_name = os::args()[0].clone().to_c_str();
        let resume = String::from_str("--resume").to_c_str();
        let windows = window_ids.to_c_str();

        let result = unsafe {
            
            let mut slice : &mut [*const i8] = [
                program_name.as_ptr(),
                resume.as_ptr(),
                windows.as_ptr(),
                null()
            ];
            execvp(filename.as_ptr(), slice.as_mut_ptr())
        };

        window_manager.clone()
    }
}
