use crate::config::Config;
use crate::core::workspaces::Workspaces;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::RwLock;
use crate::window_system::*;

#[macro_export]
macro_rules! add_key_handler_str(
    ($config: expr, $w:expr, $key:expr, $modkey:expr, $inp:expr) => (
        $config.add_key_handler($w.get_keycode_from_string($key), $modkey, Box::new($inp));
    )
);

#[macro_export]
macro_rules! add_key_handler_code(
    ($config: expr, $key:expr, $modkey:expr, $inp:expr) => (
        $config.add_key_handler($key, $modkey, Box::new($inp));
    )
);

#[macro_export]
macro_rules! add_mouse_handler(
    ($config: expr, $button:expr, $modkey:expr, $inp:expr) => (
        $config.add_mouse_handler($button, $modkey, Box::new($inp));
    )
);

#[macro_export]
macro_rules! send_layout_message(
    ($message: expr) => (
        |m, w, c| m.send_layout_message($message, w.deref(), c).windows(w.deref(), c, &|x| x.clone())
    )
);

#[macro_export]
macro_rules! run(
    ($command: expr, $options: expr) => (
        |w, _, _| { run($command, String::from($options).split(' ').map(String::from).collect()); w }
    )
);

pub fn run<S: AsRef<OsStr>>(program: S, args: Vec<String>) {
    Command::new(program).args(&args).spawn().unwrap();
}

pub fn spawn_pipe<S: AsRef<OsStr>>(
    config: &mut Config,
    program: S,
    args: Vec<String>,
) -> Rc<RwLock<Child>> {
    let result = Command::new(program)
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let rc = Rc::new(RwLock::new(result));
    config.general.pipes.push(rc.clone());
    rc
}

pub fn spawn_on(
    workspaces: Workspaces,
    _: &dyn WindowSystem,
    window: Window,
    workspace_id: u32,
) -> Workspaces {
    workspaces.focus_window(window).shift(workspace_id)
}
