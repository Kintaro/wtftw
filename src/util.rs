use std::io::process::Command;
use std::io::process::Process;

pub fn spawn_pipe(program: String, args: Option<String>) -> Process {
    match args {
        Some(a) => Command::new(program).arg(a).detached().spawn().unwrap(),
        None    => Command::new(program).detached().spawn().unwrap()
    }
}

pub fn spawn_on(program: String, args: Option<String>, workspace_id: u32) {
}
