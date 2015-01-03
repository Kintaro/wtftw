#![feature(unboxed_closures)]
#![feature(phase)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(default_type_params)]
#![feature(old_orphan_check)]
#[phase(plugin, link)]
extern crate log;
extern crate serialize;

pub mod config;
pub mod core;
pub mod handlers;
pub mod layout;
pub mod logger;
pub mod util;
pub mod window_manager;
pub mod window_system;
