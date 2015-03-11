use std::process::Command;
use std::process::Child;
use std::rc::Rc;
use std::sync::RwLock;
use std::ffi::AsOsStr;
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

pub fn run<S: AsOsStr + ?Sized>(program: &S, args: Vec<String>) {
    match Command::new(program).args(&args).spawn() {
        _ => ()
    }
}

pub fn spawn_pipe<S: AsOsStr + ?Sized>(config: &mut Config, program: &S, args: Vec<String>) -> Rc<RwLock<Child>> {
    let result = Command::new(program)
        .args(&args).spawn().unwrap();
    let rc = Rc::new(RwLock::new(result));
    config.general.pipes.push(rc.clone());
    rc
}

pub fn spawn_on<'a>(workspaces: Workspaces<'a>, _: &WindowSystem,
                window: Window, workspace_id: u32) -> Workspaces<'a> {
    workspaces.focus_window(window).shift(workspace_id)
}
