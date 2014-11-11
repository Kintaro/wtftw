use window_manager::WindowManager;
use window_system::WindowSystem;
use window_system::Window;
use config::Config;

#[deriving(Clone)]
pub type KeyHandler<'a> = Box<Fn<(WindowManager, &'a WindowSystem + 'a, &'a Config<'a>), WindowManager> + 'static>;pub type ManageHook<'a> = Box<Fn<(WindowManager, &'a WindowSystem + 'a, &'a Config<'a>, Window), 
    WindowManager> + 'static>;

/// Some default handlers for easier config scripts
pub mod default {
    use std::io::process::Command;
    use window_manager::WindowManager;
    use window_system::WindowSystem;
    use config::Config;

    pub fn start_terminal(window_manager: WindowManager, _: &WindowSystem, 
                          config: &Config) -> WindowManager {
        let (terminal, args) = config.terminal.clone();
        let arguments : Vec<String> = args.split(' ').map(String::from_str).collect();
        spawn(proc() {
            debug!("spawning terminal");
            Command::new(terminal).args(arguments.as_slice()).detached().spawn();
        });

        window_manager.clone()
    }

    pub fn start_launcher(window_manager: WindowManager, window_system: &WindowSystem, 
                          config: &Config) -> WindowManager {
        let launcher = config.launcher.clone();
        spawn(proc() {
            debug!("spawning launcher");
            Command::new(launcher).detached().spawn();
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
}
