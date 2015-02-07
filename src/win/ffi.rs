extern crate winapi;
extern crate "kernel32-sys" as win;

pub use self::winapi::{
	INVALID_HANDLE_VALUE,
	HANDLE,
	BOOL,
	LPCWSTR,
	ULONG_PTR,
	DWORD,
};

use std::ffi::AsOsStr;
use std::os::windows::OsStrExt;
// @todo, change to std::io once stable.
use std::old_io as io;
use std::error::FromError;

use fsnotify::*;

/**
 * Converts a Path to a LPCWSTR (*const wchar_t) for use in FFI.
 */
pub fn path_to_lpcwstr( path: &Path ) -> LPCWSTR {
	// https://gist.github.com/aturon/9e2f4365c3a01684685c
	// https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/ext.rs#L115-L117
	let mut buf: Vec<u16> = path.as_os_str().encode_wide().collect();
	buf.push( 0 ); // null terminator
	return buf.as_ptr();
}

/**
 * Takes the last error and produces an Error:Io (R) from it.
 */
pub unsafe fn last_error<T>() -> NotifyResult<T> {
	Err( Error::Io( io::IoError::from_errno( win::GetLastError() as usize, true ) ) )
}

pub unsafe fn create_io_completion_port( file_handle: HANDLE, existing_port_completeion: HANDLE ) -> HANDLE {
	return win::CreateIoCompletionPort( file_handle, existing_port_completeion, 0 as ULONG_PTR, 0 as DWORD );
}
