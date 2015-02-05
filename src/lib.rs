#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// https://github.com/rust-lang/rust/issues/17858
#![feature(unsafe_destructor)]

// Required by operations.rs
#![feature(hash)]
#[macro_use] extern crate bitflags;

mod fsnotify;
pub use self::fsnotify::*;

#[cfg(target_os="linux")]
pub mod linux;

#[cfg(target_os="osx")]
pub mod osx;

#[cfg(target_os="windows")]
pub mod win;