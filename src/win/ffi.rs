//================================================================================
// Windows API imports:
//================================================================================

extern crate winapi;
extern crate "kernel32-sys" as kernel32;

// All types used in winapi that are used in win/mod.rs.
pub use self::winapi::{
	INVALID_HANDLE_VALUE,
	LPOVERLAPPED,
	ULONG_PTR,
	HANDLE,
	DWORD,
	BOOL,
};

// All types used in winapi.
use self::winapi::{
	LPCWSTR,
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
	PostQueuedCompletionStatus,
	CreateIoCompletionPort,
	GetLastError,
	CreateFileW,
};

//================================================================================
// Other imports:
//================================================================================

use std::ffi::AsOsStr;
use std::os::windows::OsStrExt;

use std::io;
use std::error::FromError;

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
		return {
			let $v = unsafe { $call };
			if $v == $error { last_error() }
			else { Ok( $r ) }
		}
	};
	( $error: expr, $call: expr ) => { win_guard!( a, a, $error, $call ) };
	( $call:expr ) => { win_guard!( INVALID_HANDLE_VALUE, $call ) };
}

macro_rules! win_bool {
	( $call: expr ) => { win_guard!( v, (), 0 as BOOL, $call ) };
}

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

pub fn post_queued_completion_status( port: HANDLE ) -> R {
	win_bool!( PostQueuedCompletionStatus( port, 0 as DWORD, 0 as ULONG_PTR, 0 as LPOVERLAPPED ) );
}

pub fn create_io_completion_port( file_handle: HANDLE, existing_port_completeion: HANDLE ) -> NotifyResult<HANDLE> {
	win_guard!( CreateIoCompletionPort( file_handle, existing_port_completeion, 0 as ULONG_PTR, 0 as DWORD ) );
}

pub fn file_handle( path: &Path ) -> NotifyResult<HANDLE> {
	let p = path_to_utf16ptr( path );
	win_guard!( CreateFileW( p,
		FILE_LIST_DIRECTORY,
		FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
		0 as LPSECURITY_ATTRIBUTES,
		OPEN_EXISTING,
		FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED as DWORD,
		0 as HANDLE
	) );
}