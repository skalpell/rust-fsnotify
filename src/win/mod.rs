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

use std::path::{AsPath, Path, PathBuf};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::{Thread, JoinGuard};
use std::collections::HashMap;
use std::option::Option;
use std::marker::Send;

use fsnotify::*;
use helpers::*;

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

use self::Instruction::*;

type InsTx = mpsc::Sender<Instruction>;
type InsRx = mpsc::Receiver<Instruction>;

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
	port:		HANDLE,

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
				let mut close: bool = false;

				match ins_rx.try_recv() {
					Err( mpsc::TryRecvError::Disconnected ) => {
						// Disconnected? Will be impossible to recieve Quit, so we must close.
						close = true;
						self.is_closed = true;
					},
					Ok( Close ) => {
						// Bail! This is the main exit point, write to open to close.
						close = true;
					},
					Ok( Watch( path ) ) => {
						WinFsNotifier::add_watch();
					},
					Ok( Unwatch( path ) ) => {
						WinFsNotifier::rem_watch();
					},
					_ => {}
				}

				if close {

				}

				// Are we using recursion?
				//let recurse = self.config.is_recursive() as BOOL;
			}
		} ) );
	}

	fn add_watch() {

	}

	fn rem_watch() {

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

	/**
	 * Sends instruction to thread loop.
	 */
	fn instruct( &self, ins: Instruction ) -> R {
		try!( self.ins_tx.send( ins ) );
		post_queued_completion_status( self.port )
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
			port:		port,
		//	paths:		Arc::new( RwLock::new( HashMap::new() ) ),
		};

		n.run( ins_rx );
		Ok( n )
	}

	fn watch<P: AsPath + ?Sized>( &mut self, path: &P ) -> R {
		// Send watch signal.
		try!( self.enforce_open() );
		self.instruct( Watch( path_buf( path ) ) )
	}

	fn unwatch<P: AsPath + ?Sized>( &mut self, path: &P ) -> R {
		// Send unwatch signal.
		try!( self.enforce_open() );
		self.instruct( Unwatch( path_buf( path ) ) )
	}

	fn close( &mut self ) -> R {
		try!( self.enforce_open() );

		// Consider us closed, don't want this called again.
		self.is_closed = true;

		// Send Close signal.
		try!( self.instruct( Close ) );

		// Block/Join until thread terminates.
		return Ok( match self.join_guard.take() {
			Some( jg )	=> try!( jg.join() ),
			None		=> ()
		} );
	}

	fn is_closed( &self ) -> NotifyResult<bool> {
		Ok( self.is_closed )
	}
}
