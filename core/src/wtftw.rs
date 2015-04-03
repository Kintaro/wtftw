#![feature(unboxed_closures)]
#![feature(box_syntax)]
#![feature(libc)]
#![feature(core)]
#![feature(collections)]
#![feature(std_misc)]
#![feature(rustc_private)]
#![feature(path_ext)]
#![feature(slice_patterns)]
#[deny(warnings)]
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
pub mod util;
pub mod window_manager;
pub mod window_system;
