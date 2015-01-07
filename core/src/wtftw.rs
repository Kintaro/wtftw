#![feature(unboxed_closures)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(default_type_params)]
#![feature(old_orphan_check)]
#[macro_use]
#[link]
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
