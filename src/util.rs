use std::io::process::Command;
use std::io::process::Process;
use core::Workspaces;
use window_system::WindowSystem;
use window_system::Window;

pub fn run(program: String, args: Option<String>) {
    let arguments : Vec<String> = match args {
        None => Vec::new(),
        Some(ref a) => a.split(' ').map(String::from_str).collect()
    };

    match Command::new(program).args(arguments.as_slice()).detached().spawn() {
        _ => ()
    }
}

pub fn spawn_pipe(program: String, args: Option<String>) -> Process {
    match args {
        Some(a) => Command::new(program).arg(a).detached().spawn().unwrap(),
        None    => Command::new(program).detached().spawn().unwrap()
    }
}

pub fn spawn_on(workspaces: Workspaces, _: &WindowSystem,
                window: Window, workspace_id: u32) -> Workspaces {
    workspaces.focus_window(window).shift(workspace_id)
}
