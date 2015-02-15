#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// @TODO remove!
#![allow(dead_code, unused_imports, unused_variables)]

#![feature(io, path)]

// Required by windows encode_wide().
#![feature(std_misc)]

// Required by operations.rs
#![feature(hash)]
#[macro_use] extern crate bitflags;

//================================================================================
// helper macros:
//================================================================================

#[macro_use]
mod macros;

//================================================================================
// fsnotify interface:
//================================================================================

mod fsnotify;
pub use self::fsnotify::*;

//================================================================================
// concrete platform implementations:
//================================================================================

#[cfg(target_os="linux")]
pub mod linux;
#[cfg(target_os="linux")]
pub type RecommendedNotifier<'a> = linux::LinuxFsNotifier<'a>;

#[cfg(target_os="macos")]
pub mod osx;
#[cfg(target_os="macos")]
pub type RecommendedNotifier<'a> = osx::OsxFsNotifier<'a>;

#[cfg(target_os="windows")]
pub mod win;
#[cfg(target_os="windows")]
pub type RecommendedNotifier<'a> = win::WinFsNotifier<'a>;

//================================================================================
// generic interface:
//================================================================================

pub fn new<'a>( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<RecommendedNotifier<'a>> {
	FsNotifier::new( sender, config )
}