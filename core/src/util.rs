use std::io::process::Command;
use std::io::process::Process;
use std::rc::Rc;
use std::sync::RWLock;
use core::Workspaces;
use config::Config;
use window_system::*;

#[macro_export]
macro_rules! add_key_handler_str(
    ($config: expr, $w:expr, $key:expr, $modkey:expr, $inp:expr) => (
        $config.add_key_handler($w.get_keycode_from_string($key), $modkey, box $inp);
    )
);

#[macro_export]
macro_rules! add_key_handler_code(
    ($config: expr, $key:expr, $modkey:expr, $inp:expr) => (
        $config.add_key_handler($key, $modkey, box $inp);
    )
);

#[macro_export]
macro_rules! add_mouse_handler(
    ($config: expr, $button:expr, $modkey:expr, $inp:expr) => (
        $config.add_mouse_handler($button, $modkey, box $inp);
    )
);


pub fn run(program: &str, args: Option<&str>) {
    let arguments : Vec<String> = match args {
        None => Vec::new(),
        Some(a) => String::from_str(a).split(' ').map(String::from_str).collect()
    };

    debug!("trying to run {} {}", program, arguments);

    match Command::new(String::from_str(program)).args(arguments.as_slice()).detached().spawn() {
        _ => ()
    }
}

pub fn spawn_pipe(config: &mut Config, program: String, args: Option<String>) -> Rc<RWLock<Process>> {
    let result = match args {
        Some(a) => Command::new(program).arg(a).detached().spawn().unwrap(),
        None    => Command::new(program).detached().spawn().unwrap()
    };
    debug!("Created pipe with id {}", result.id());
    let rc = Rc::new(RWLock::new(result));
    config.general.pipes.push(rc.clone());
    rc
}

pub fn spawn_on<'a>(workspaces: Workspaces<'a>, _: &WindowSystem,
                window: Window, workspace_id: u32) -> Workspaces<'a> {
    workspaces.focus_window(window).shift(workspace_id)
}
