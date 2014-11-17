#![feature(unboxed_closures, unboxed_closure_sugar, overloaded_calls)]
#![feature(phase)]
#![feature(globs)]
#[phase(plugin, link)]
extern crate log;
extern crate serialize;

use config::Config;
use handlers::default::*;
use window_manager::WindowManager;
use window_system::*;

pub mod config;
pub mod core;
pub mod handlers;
pub mod layout;
pub mod logger;
pub mod window_manager;
pub mod window_system;
pub mod xlib_window_system;

include!("local_config.rs")
