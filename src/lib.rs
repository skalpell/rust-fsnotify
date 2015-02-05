#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// Required by operations.rs
#![feature(hash)]
#[macro_use] extern crate bitflags;

mod fsnotify;
pub use self::fsnotify::*;