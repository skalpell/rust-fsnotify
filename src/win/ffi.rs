//================================================================================
// Windows API imports:
//================================================================================

extern crate winapi;
extern crate kernel32;

// All types used in winapi that are used in win/mod.rs.
pub use self::winapi::{
	OVERLAPPED,
	LPOVERLAPPED,
};

// All types and constants used in winapi.
use self::winapi::{
	INFINITE,
	DWORD,
	BOOL,
	LPVOID,
	LPOVERLAPPED_COMPLETION_ROUTINE,
	LPBY_HANDLE_FILE_INFORMATION,
	BY_HANDLE_FILE_INFORMATION,
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
	CloseHandle,
	CancelIo,
	GetFileInformationByHandle,
//	ReadDirectoryChangesW,
};

// Temporary:
extern {
	fn ReadDirectoryChangesW(
		hDirectory: WIN_HANDLE, lpBuffer: LPVOID, nBufferLength: DWORD, bWatchSubtree: BOOL,
		dwNotifyFilter: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED,
		lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
	) -> BOOL;
}

//================================================================================
// Fixing HANDLE w.r.t Send:
//================================================================================

use self::winapi::HANDLE as WIN_HANDLE;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct HANDLE( WIN_HANDLE );
unsafe impl Send for HANDLE {}
unsafe impl Sync for HANDLE {}

pub const INVALID_HANDLE: HANDLE = HANDLE( INVALID_HANDLE_VALUE );
pub const NULL_HANDLE: HANDLE = HANDLE( 0 as WIN_HANDLE );

//================================================================================
// Other imports:
//================================================================================

use std::os::windows::ffi::OsStrExt;
use std::path::{Path};
use std::io;
use std::ptr::null_mut;
use std::mem;

use fsnotify::*;

//================================================================================
// Error handling:
//================================================================================

/**
 * Takes the last error and produces an Error:Io (R) from it.
 */
pub fn last_error<T>() -> NotifyResult<T> {
	Err( From::from( io::Error::last_os_error() ) )
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

// see https://github.com/howeyc/fsnotify/issues/49
pub const ERROR_MORE_DATA:			i32	= 234;
pub const ERROR_ACCESS_DENIED:		i32	= 5;
pub const ERROR_OPERATION_ABORTED:	i32	= 995;

//================================================================================
// FFI: Completion ports:
//================================================================================

pub fn get_queued_completion_status<'a>( HANDLE( port ): HANDLE ) -> (R, u32, LPOVERLAPPED) {
	let n: LPDWORD = null_mut();
	let key: PULONG_PTR = null_mut();
	let ov: *mut LPOVERLAPPED = null_mut();
	(
		win_bool!( GetQueuedCompletionStatus( port, n, key, ov, INFINITE ) ),
		unsafe { *n },
		unsafe { *ov }
	)
}

pub fn post_queued_completion_status( HANDLE( port ): HANDLE ) -> R {
	win_bool!( PostQueuedCompletionStatus( port, 0 as DWORD, 0 as ULONG_PTR, null_mut() ) )
}

pub fn create_io_completion_port( HANDLE( file_handle ): HANDLE, HANDLE( existing_port_completeion ): HANDLE ) -> NotifyResult<HANDLE> {
	win_handle!( CreateIoCompletionPort( file_handle, existing_port_completeion, 0 as ULONG_PTR, 0 as DWORD ) )
}

//================================================================================
// FFI: File IO:
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

fn get_file_information_by_handle( HANDLE( file ) : HANDLE ) -> NotifyResult<BY_HANDLE_FILE_INFORMATION> {
	let fi: LPBY_HANDLE_FILE_INFORMATION = null_mut();
	win_guard!( v, unsafe { *fi }, 0 as BOOL, GetFileInformationByHandle( file, fi ) )
}

pub fn close_handle( HANDLE( handle ) : HANDLE ) -> R {
	win_bool!( CloseHandle( handle ) )
}

pub fn cancel_io( HANDLE( file ) : HANDLE ) -> R {
	win_bool!( CancelIo( file ) )
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

pub struct INode {
    pub handle:	HANDLE,
    pub volume:	u32,
    pub index:	u64,
}

pub fn file_inode( path: &Path ) -> NotifyResult<INode> {
	let h = try!( file_handle( path ) );
	match get_file_information_by_handle( h ) {
		Ok( fi ) => {
			Ok( INode {
				handle:	h,
				volume:	fi.dwVolumeSerialNumber,
				index:	(fi.nFileIndexHigh as u64) << 32 | (fi.nFileIndexLow as u64)
			} )
		},
		Err( err ) => {
			try!( close_handle( h ) );
			Err( err )
		}
	}
}

pub fn read_directory_changes(
	HANDLE( dir ): HANDLE, buffer: &mut [u8],
	watch_subtree: bool, notify_filter: DWORD, overlapped: LPOVERLAPPED ) -> R {

	// These are not used:
	let bytes_returned = null_mut();
	let completion_routine = None;

	// Convert to buffer ptr & length.
	let (buf, buf_len) = (buffer.as_mut_ptr() as LPVOID, buffer.len() as DWORD);

	win_bool!( ReadDirectoryChangesW(
		dir,
		buf, buf_len,
		watch_subtree as BOOL,
		notify_filter,
		bytes_returned,
		overlapped,
		completion_routine
	) )
}

pub fn make_overlapped() -> OVERLAPPED {
	unsafe { mem::uninitialized() }
}