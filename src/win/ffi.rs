extern crate winapi;
extern crate "kernel32-sys" as kernel32;

// All types used in winapi that are used in win/mod.rs.
pub use self::winapi::{
	INVALID_HANDLE_VALUE,
	HANDLE,
};

// All types used in winapi.
use self::winapi::{
	BOOL,
	LPCWSTR,
	ULONG_PTR,
	DWORD,
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
//	FILE_FLAG_BACKUP_SEMANTICS,
//	FILE_FLAG_OVERLAPPED,
	/* end: CreateFile */
};

// @TODO remove once in winapi.
const FILE_FLAG_BACKUP_SEMANTICS: DWORD	= 0x02000000;
const FILE_FLAG_OVERLAPPED: DWORD		= 0x40000000;

// All functions used in winapi/kernel32.
use self::kernel32::{
	CreateIoCompletionPort,
	GetLastError,
	CreateFileW,
};

use std::ffi::AsOsStr;
use std::os::windows::OsStrExt;
// @todo, change to std::io once stable.
use std::old_io as io;
use std::error::FromError;

use fsnotify::*;

/**
 * Takes the last error and produces an Error:Io (R) from it.
 */
pub unsafe fn last_error<T>() -> NotifyResult<T> {
	Err( Error::Io( io::IoError::from_errno( GetLastError() as usize, true ) ) )
}

macro_rules! win_guard {
	( $call:expr ) => {
		{
			unsafe {
				let a = $call;
				if a == INVALID_HANDLE_VALUE { last_error() } else { Ok( a ) }
			}
		};
	}
}

/**
 * Converts a Path to a LPCWSTR (*const wchar_t) for use in FFI.
 */
pub fn path_to_utf16ptr( path: &Path ) -> LPCWSTR {
	// https://gist.github.com/aturon/9e2f4365c3a01684685c
	// https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/ext.rs#L115-L117
	let mut buf: Vec<u16> = path.as_os_str().encode_wide().collect();
	buf.push( 0 ); // null terminator
	return buf.as_ptr();
}

pub fn create_io_completion_port( file_handle: HANDLE, existing_port_completeion: HANDLE ) -> NotifyResult<HANDLE> {
	return win_guard!( CreateIoCompletionPort( file_handle, existing_port_completeion, 0 as ULONG_PTR, 0 as DWORD ) );
}

pub fn file_handle( path: &Path ) -> NotifyResult<HANDLE> {
	let p = path_to_utf16ptr( path );
	return win_guard!( CreateFileW( p,
		FILE_LIST_DIRECTORY,
		FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
		0 as LPSECURITY_ATTRIBUTES,
		OPEN_EXISTING,
		FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED as DWORD,
		0 as HANDLE
	) );
}