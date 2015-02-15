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
use std::sync::{Arc, RwLock};
use std::thread::Thread;
//use std::marker::Send;

use fsnotify::*;

mod ffi;
use self::ffi::*;

pub struct WinFsNotifier<'a> {
	config:	Configuration<'a>,
	sender:	EventSender,

	// Indicates if the notifier is open or not, setting it to false closes it:
	open: Arc<RwLock<bool>>,

	// Add and remove operations queue:
	queue:	Arc<RwLock<Vec<Instruction>>>,

	// Handle to completion port:
	port:	HANDLE,

//	paths:	Arc<RwLock<HashMap<HANDLE, PathBuf>>>,
}

fsn_drop!(	WinFsNotifier );

enum InsType {
    ADD, REMOVE
}

struct Instruction {
    instruction: InsType,
    path: PathBuf,
}

impl<'a> WinFsNotifier<'a> {
	fn run( &mut self ) {
		// Clone stuff.
		let (open, tx) = (
			self.open.clone(),
			self.sender.clone()
		);

		// Run notifier in thread.
		Thread::spawn( move || {
			loop {
				if !(*open.read().unwrap()) {
					break
				}

				// Are we using recursion?
				//let recurse = self.config.is_recursive() as BOOL;
			}
		} );
	}
}

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
	fsn_close!();

	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self> {
		// Retrieve IOCP.
		let port = try!( create_io_completion_port( INVALID_HANDLE_VALUE, 0 as HANDLE ) );

		fsn_run!( WinFsNotifier {
			config: config,
			sender: sender,

			open:	Arc::new( RwLock::new( true ) ),
			port:	port,
		//	paths:	Arc::new( RwLock::new( HashMap::new() ) ),

			queue:	Arc::new( RwLock::new( vec![] ) ),
		} )
	}

	fn watch( &mut self, path: FilePath ) -> R {
		// Add to queue, handle in start().
	//	self.add_queue.push( path.to_path_buf() );

		not_implemented!();
	}

	fn unwatch( &mut self, path: FilePath ) -> R {
		not_implemented!();
	}
}
