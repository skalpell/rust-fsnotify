#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// @TODO remove!
#![allow(dead_code, unused_variables)]

#[macro_use]
extern crate bitflags;

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
pub type RecommendedNotifier = win::WinFsNotifier;

//================================================================================
// generic interface:
//================================================================================

pub fn new( sender: EventSender, config: Configuration ) -> NotifyResult<RecommendedNotifier> {
	FsNotifier::new( sender, config )
}