use std::old_io::process::Command;
use std::old_io::process::Process;
use std::rc::Rc;
use std::sync::RwLock;
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

#[macro_export]
macro_rules! send_layout_message(
    ($message: expr) => (
        |m, w, c| m.send_layout_message($message, w, c).windows(w, c, |x| x.clone())
    )
);

#[macro_export]
macro_rules! run(
    ($command: expr, $options: expr) => (
        |w, _, _| { run($command, String::from_str($options).split(' ').map(String::from_str).collect()); w }
    )
);

pub fn run(program: &str, args: Vec<String>) {
    debug!("trying to run {}", program);
    match Command::new(String::from_str(program)).args(&args).detached().spawn() {
        _ => ()
    }
}

pub fn spawn_pipe(config: &mut Config, program: &str, args: Vec<String>) -> Rc<RwLock<Process>> {
    let result = Command::new(String::from_str(program))
        .args(&args).detached().spawn().unwrap();
    debug!("Created pipe with id {}", result.id());
    let rc = Rc::new(RwLock::new(result));
    config.general.pipes.push(rc.clone());
    rc
}

pub fn spawn_on<'a>(workspaces: Workspaces<'a>, _: &WindowSystem,
                window: Window, workspace_id: u32) -> Workspaces<'a> {
    workspaces.focus_window(window).shift(workspace_id)
}
