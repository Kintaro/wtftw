#![feature(unboxed_closures)]
#![feature(old_orphan_check)]
#![feature(box_syntax)]
#[macro_use]
#[link]
extern crate log;
#[macro_use] 
extern crate bitflags;
extern crate serialize;

pub mod config;
pub mod core;
pub mod handlers;
pub mod layout;
pub mod logger;
pub mod util;
pub mod window_manager;
pub mod window_system;
