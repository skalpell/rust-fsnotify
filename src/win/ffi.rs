//================================================================================
// Windows API imports:
//================================================================================

extern crate winapi;
extern crate "kernel32-sys" as kernel32;

// All types used in winapi that are used in win/mod.rs.
pub use self::winapi::{
	OVERLAPPED,
};

// All types and constants used in winapi.
use self::winapi::{
	DWORD,
	BOOL,
	LPOVERLAPPED,
	ULONG_PTR,
	INVALID_HANDLE_VALUE,
	LPCWSTR,
	LPDWORD,
	PULONG_PTR,
	/* start: CreateFile */
	LPSECURITY_ATTRIBUTES,
	OPEN_EXISTING,
	// access (read / write) mode.
	FILE_LIST_DIRECTORY,
	// share mode
	FILE_SHARE_READ,
	FILE_SHARE_WRITE,
	FILE_SHARE_DELETE,
	// file attributes
	FILE_FLAG_BACKUP_SEMANTICS,
	FILE_FLAG_OVERLAPPED,
	/* end: CreateFile */
};

// All functions used in winapi/kernel32.
use self::kernel32::{
	GetQueuedCompletionStatus,
	PostQueuedCompletionStatus,
	CreateIoCompletionPort,
	CreateFileW,
};

//================================================================================
// Fixing HANDLE w.r.t Send:
//================================================================================

use self::winapi::HANDLE as WIN_HANDLE;

#[derive(Hash, Eq, PartialEq)]
pub struct HANDLE( WIN_HANDLE );
unsafe impl Send for HANDLE {}

pub const INVALID_HANDLE: HANDLE = HANDLE( INVALID_HANDLE_VALUE );
pub const NULL_HANDLE: HANDLE = HANDLE( 0 as WIN_HANDLE );

//================================================================================
// Other imports:
//================================================================================

use std::ffi::AsOsStr;
use std::os::windows::OsStrExt;

use std::io;
use std::error::FromError;

use std::ptr::{null, null_mut};

use fsnotify::*;

//================================================================================
// Error handling:
//================================================================================

/**
 * Takes the last error and produces an Error:Io (R) from it.
 */
pub fn last_error<T>() -> NotifyResult<T> {
	Err( FromError::from_error( io::Error::last_os_error() ) )
}

macro_rules! win_guard {
	( $v: ident, $r: expr, $error: expr, $call: expr ) => {
		{
			let $v = unsafe { $call };
			if $v == $error { last_error() }
			else { Ok( $r ) }
		}
	};
	( $error: expr, $call: expr ) => { win_guard!( a, a, $error, $call ) };
}
macro_rules! win_handle	{ ( $call: expr ) => { win_guard!( v, HANDLE( v ), INVALID_HANDLE_VALUE, $call ) }; }
macro_rules! win_bool	{ ( $call: expr ) => { win_guard!( v, (), 0 as BOOL, $call ) }; }

//================================================================================
// Helpers:
//================================================================================

/**
 * Converts a Path to a LPCWSTR (*const wchar_t) for use in FFI.
 */
fn path_to_utf16ptr( path: &Path ) -> LPCWSTR {
	// https://gist.github.com/aturon/9e2f4365c3a01684685c
	// https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/ext.rs#L115-L117
	let mut buf: Vec<u16> = path.as_os_str().encode_wide().collect();
	buf.push( 0 ); // null terminator
	return buf.as_ptr();
}

//================================================================================
// FFI:
//================================================================================

pub fn get_queued_completion_status<'a>( &HANDLE( port ): &HANDLE ) -> (R, u32, Option<&'a OVERLAPPED>) {
	let n: LPDWORD = null_mut();
	let key: PULONG_PTR = null_mut();
	let ov: *mut LPOVERLAPPED = null_mut();

	(
		win_bool!( GetQueuedCompletionStatus( port, n, key, ov, -1 ) ),
		unsafe { *n },
		unsafe { (*ov).as_ref() }
	)
}

pub fn post_queued_completion_status( &HANDLE( port ): &HANDLE ) -> R {
	win_bool!( PostQueuedCompletionStatus( port, 0 as DWORD, 0 as ULONG_PTR, null_mut() ) )
}

pub fn create_io_completion_port( HANDLE( file_handle ): HANDLE, HANDLE( existing_port_completeion ): HANDLE ) -> NotifyResult<HANDLE> {
	win_handle!( CreateIoCompletionPort( file_handle, existing_port_completeion, 0 as ULONG_PTR, 0 as DWORD ) )
}

pub fn file_handle( path: &Path ) -> NotifyResult<HANDLE> {
	let p = path_to_utf16ptr( path );
	win_handle!( CreateFileW( p,
		FILE_LIST_DIRECTORY,
		FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
		0 as LPSECURITY_ATTRIBUTES,
		OPEN_EXISTING,
		FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED as DWORD,
		0 as WIN_HANDLE
	) )
}