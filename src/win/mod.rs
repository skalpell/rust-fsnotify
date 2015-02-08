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
use std::sync::RwLock;
use std::sync::Arc;
use std::marker::Send;

use fsnotify::*;

mod ffi;
use self::ffi::*;

struct WinFsNotifier<'a> {
	config:	Configuration<'a>,
	sender:	EventSender<'a>,

	// Indicates if the notifier is open or not, setting it to false closes it.
	open: Arc<RwLock<bool>>,
//	paths:	Arc<RwLock<HashMap<HANDLE, PathBuf>>>,

	// Handle to completion port:
	port:	HANDLE,
	add_queue: Vec<PathBuf>,
}

fsnotify_drop!( WinFsNotifier );

impl<'a> WinFsNotifier<'a> {
	fn run( &mut self ) {

	}
}

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
	fn new( sender: EventSender<'a>, config: Configuration<'a> ) -> NotifyResult<Self> {
		// Retrieve IOCP.
		let port = try!( create_io_completion_port( INVALID_HANDLE_VALUE, 0 as HANDLE ) );

		Ok( WinFsNotifier {
			config: config,
			sender: sender,

			open:	Arc::new( RwLock::new( true ) ),
			port:	port,
		//	paths:	Arc::new( RwLock::new( HashMap::new() ) ),

			add_queue: vec![],
		} )
	}

	fn watch( &mut self, path: &Path ) -> R {
		// Are we using recursion?
		//let recurse = self.config.is_recursive() as BOOL;
		// Add to queue, handle in start().
		self.add_queue.push( path.to_path_buf() );

		not_implemented!();
	}

	fn unwatch( &mut self, path: &Path ) -> R {
		not_implemented!();
	}

	fn close( &mut self ) -> R {
		not_implemented!();
	}
}
