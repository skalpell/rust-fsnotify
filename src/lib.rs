#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// @TODO remove!
#![allow(dead_code, unused_imports, unused_variables)]

#![feature(io, path)]

// Required by windows encode_wide().
#![feature(std_misc)]

// for Send, RFC 458:
#![feature(core)]

// Required by operations.rs
#![feature(hash)]
#[macro_use] extern crate bitflags;

//================================================================================
// helpers and macros:
//================================================================================

#[macro_use]
mod helpers;

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

pub fn new<'a>( sender: EventSender, config: Configuration ) -> NotifyResult<RecommendedNotifier<'a>> {
	FsNotifier::new( sender, config )
}