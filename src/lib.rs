#![crate_type = "lib"]
#![crate_name = "fsnotify"]

// @TODO remove!
#![allow(dead_code, unused_imports, unused_variables)]

#![feature(io, path)]

// https://github.com/rust-lang/rust/issues/17858
#![feature(unsafe_destructor)]

// Required by windows encode_wide().
#![feature(std_misc)]

// Required by operations.rs
#![feature(hash)]
#[macro_use] extern crate bitflags;

//================================================================================
// fsnotify interface:
//================================================================================

mod fsnotify;
pub use self::fsnotify::*;

//================================================================================
// helper macros:
//================================================================================

macro_rules! not_implemented {
	() => { return Err( Error::NotImplemented ) }
}

macro_rules! fsnotify_drop {
	( $clazz:ident ) => {
		#[unsafe_destructor]
		impl<'a> Drop for $clazz<'a> {
			fn drop( &mut self ) {
				self.close().ok().expect( "Failed to stop" );
			}
		}
	}
}

//================================================================================
// concrete platform implementations:
//================================================================================

#[cfg(target_os="linux")]
pub mod linux;

#[cfg(target_os="osx")]
pub mod osx;

#[cfg(target_os="windows")]
pub mod win;