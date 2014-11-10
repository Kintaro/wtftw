#![feature(unboxed_closures, unboxed_closure_sugar, overloaded_calls)]
use window_manager::WindowManager;
use window_system::WindowSystem;
use config::Config;

#[deriving(Clone)]
pub type KeyHandler<'a> = Box<Fn<(WindowManager, &'a WindowSystem + 'a, &'a Config<'a>), WindowManager> + 'static>;

/// Some default handlers for easier config scripts
pub mod default {
    use std::io::process::Command;
    use window_manager::WindowManager;
    use window_system::WindowSystem;
    use config::Config;
    use handlers::KeyHandler;

    pub fn start_terminal(window_manager: WindowManager, window_system: &WindowSystem, 
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
        let mut local_window_manager = window_manager.clone();

        debug!("switching to workspace {}", config.tags[index].clone());
        local_window_manager.view(window_system, index as u32, config);

        local_window_manager
    }

    pub fn move_window_to_workspace(window_manager: WindowManager, window_system: &WindowSystem, 
                                    config: &Config, index: uint) -> WindowManager {
        let mut local_window_manager = window_manager.clone();
        let focused_window = window_system.get_focused_window();

        debug!("moving window to workspace {}", 
               config.tags[index].clone());

        local_window_manager.view(window_system, index as u32, config);

        local_window_manager
    }

}
