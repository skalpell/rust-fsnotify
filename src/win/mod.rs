extern crate winapi;
extern crate "kernel32-sys" as win;

/**
 * Compulsory readup:
 * 1. http://qualapps.blogspot.se/2010/05/understanding-readdirectorychangesw.html
 * 2. http://stackoverflow.com/questions/339776/asynchronous-readdirectorychangesw
 * 3. https://msdn.microsoft.com/en-us/library/windows/desktop/aa363862%28v=vs.85%29.aspx
 * 4. https://msdn.microsoft.com/en-us/library/windows/desktop/aa365465%28v=vs.85%29.aspx
 *
 * The Windows implementation uses IOCP, I/O Completion Port(s) since it is regarded as most performant (read 1).
 * FindFirstChangeNotification API is **NOT** used.
 */

use std::collections::HashMap;
use std::path::PathBuf;

use fsnotify::*;

mod ffi;
use self::ffi::*;

macro_rules! win_guard_handle {
	( $call:expr ) => {
		{
			unsafe {
				let a: HANDLE = $call;
				if a == INVALID_HANDLE_VALUE {
					return last_error();
				}
				a
			}
		};
	}
}

struct WinFsNotifier<'a> {
	config:	Configuration<'a>,
	sender:	EventSender<'a>,

	started: bool,
	// add(...) adds into this "queue".
	add_queue: Vec<PathBuf>,

	// Handle to completion port:
	//port:	HANDLE,

	//paths:	HashMap<HANDLE, Path>,
}

fsnotify_drop!( WinFsNotifier );

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
	fn new( sender: EventSender<'a>, config: Configuration<'a> ) -> Self {
		WinFsNotifier {
			config: config,
			sender: sender,

			started: false,
			add_queue: vec![],
		//	port: port,
		}
	}

	fn add( &mut self, path: &Path ) -> R {
		if self.started {
			/*
			// Convert path.
			let cpath: LPCWSTR = path_to_lpcwstr( path );

			// Are we using recursion?
			let recurse = self.config.is_recursive() as BOOL;
			*/
		} else {
			// Add to queue, handle in start().
			self.add_queue.push( path.to_path_buf() );
		}

		not_implemented!();
	}

	fn remove( &mut self, path: &Path ) -> R {
		not_implemented!();
	}

	fn start( &mut self ) -> R {
		let port = win_guard_handle!( create_io_completion_port( INVALID_HANDLE_VALUE, 0 as HANDLE ) );
		not_implemented!();
	}

	fn stop( &mut self ) -> R {
		not_implemented!();
	}
}
