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
use std::path::{AsPath, Path, PathBuf};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::{Thread, JoinGuard};
use std::option::Option;
//use std::marker::Send;

use fsnotify::*;

mod ffi;
use self::ffi::*;

//================================================================================
// Enum for sending Close/Watch/Unwatch instructions:
//================================================================================

enum Instruction {
	Close,
	Watch( PathBuf ),
	Unwatch( PathBuf ),
}
type InsTx		= mpsc::Sender<Instruction>;
type InsRx		= mpsc::Receiver<Instruction>;

//================================================================================
// Actual struct:
//================================================================================

pub struct WinFsNotifier<'a> {
	config:		Configuration<'a>,
	event_tx:	EventSender,
	join_guard: Option<JoinGuard<'a, ()>>,

	// Indicates if the notifier is closed or not.
	is_closed:	bool,

	// Close/Watch/Unwatch instruction channel.
	ins_tx:		InsTx,

	// Handle to completion port:
//	port:		HANDLE,

//	paths:	Arc<RwLock<HashMap<HANDLE, PathBuf>>>,
}

fsn_drop!(	WinFsNotifier );

impl<'a> WinFsNotifier<'a> {
	fn run( &mut self, ins_rx: InsRx ) {
		// Clone stuff.
		let event_tx = self.event_tx.clone();

		// Run notifier in thread.
		self.join_guard = Some(
			Thread::scoped( move || {
			loop {
				match ins_rx.try_recv() {
					Err( err ) => {
						// Disconnected? Will be impossible to recieve Quit, so we must close.
						if err == mpsc::TryRecvError::Disconnected {
					//		self.is_closed = false;
							return
						}
					},
					// Bail! This is the main exit point, write to open to close.
					Ok( Instruction::Close ) => return,
					Ok( Instruction::Watch( path ) ) => (),
					Ok( Instruction::Unwatch( path ) ) => (),
				}

				// Are we using recursion?
				//let recurse = self.config.is_recursive() as BOOL;
			}
		} ) );
	}

	/**
	 * Returns `Ok` only if open.
	 */
	fn enforce_open( &self ) -> R {
		return match self.is_closed() {
			Ok( true )	=> Ok( () ),
			_			=> Err( Error::Closed )
		}
	}
}

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
	fn new( event_tx: EventSender, config: Configuration<'a> ) -> NotifyResult<Self> {
		// Retrieve IOCP.
		let port = try!( create_io_completion_port( INVALID_HANDLE_VALUE, 0 as HANDLE ) );

		let (ins_tx, ins_rx) = mpsc::channel();

		let mut n = WinFsNotifier {
			config:		config,
			event_tx:	event_tx,
			join_guard:	None,
			is_closed:	false,
			ins_tx:		ins_tx,
	//		port:		port,
		//	paths:		Arc::new( RwLock::new( HashMap::new() ) ),
		};
		n.run( ins_rx );
		Ok( n )
	}

	fn watch( &mut self, path: &AsPath ) -> R {
		try!( self.enforce_open() );

		// Add to queue, handle in start().
	//	self.add_queue.push( path.to_path_buf() );

		not_implemented!();
	}

	fn unwatch( &mut self, path: &AsPath ) -> R {
		try!( self.enforce_open() );

		not_implemented!();
	}

	fn close( &mut self ) -> R {
		try!( self.enforce_open() );

		// Consider us closed, don't want this called again.
		self.is_closed = true;

		// Send Close signal and block/join.
		try!( self.ins_tx.send( Instruction::Close ) );

		if let Some( jg ) = self.join_guard {
			return Ok( try!( jg.join() ) )
		}

		return Ok( () );
	}

	fn is_closed( &self ) -> NotifyResult<bool> {
		Ok( self.is_closed )
	}
}
