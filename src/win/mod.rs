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
// The struct:
//================================================================================

pub struct WinFsNotifier<'a> {
	/// Indicates if the notifier is closed or not.
	is_closed:	bool,

	/// Join guard, this is held until Drop.
	join_guard:	Option<JoinGuard<'a, ()>>,

	/// Close/Watch/Unwatch instruction TX.
	ins_tx:		InsTx,

	/// Handle to completion port:
	port:		Arc<HANDLE>,
}

fsn_drop!(	WinFsNotifier );

//================================================================================
// Implementation:
//================================================================================

struct ThreadState<'a> {
	/// The configuration.
	config:		Configuration,

	/// Event notification TX.
	event_tx:	EventSender,

	/// Close/Watch/Unwatch instruction RX.
	ins_rx:		InsRx,

	/// Maps file Handle -> Path.
	paths:		HashMap<HANDLE, PathBuf>,
}

impl<'a> FsNotifier for WinFsNotifier<'a> {
	fn new( event_tx: EventSender, config: Configuration ) -> NotifyResult<Self> {
		let (ins_tx, ins_rx) = mpsc::channel();

		// Retrieve IOCP.
		let port = try!( create_io_completion_port( INVALID_HANDLE, NULL_HANDLE ) );

		let mut n = WinFsNotifier {
			join_guard:	None,
			is_closed:	false,
			ins_tx:		ins_tx,
			port:		Arc::new( port ),
		};

		// Start thread.
		n.run( ThreadState {
			config:		config,
			event_tx:	event_tx,
			ins_rx:		ins_rx,
			paths:		HashMap::new(),
		} );

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

impl<'a> WinFsNotifier<'a> {
	/**
	 * Sends instruction to thread loop.
	 * Called in client thread.
	 */
	fn instruct( &self, ins: Instruction ) -> R {
		try!( self.ins_tx.send( ins ) );
		post_queued_completion_status( &self.port )
	}

	/**
	 * Returns `Ok` only if open.
	 * Called in client thread.
	 */
	fn enforce_open( &self ) -> R {
		return match self.is_closed() {
			Ok( true )	=> Ok( () ),
			_			=> Err( Error::Closed )
		}
	}

	fn run( &mut self, ts: ThreadState ) {
		// Clone Arc:s.
		let (is_closed, port) = (
			self.is_closed.clone(),
			self.port.clone()
		);

		// Run notifier in thread.
		self.join_guard = Some( Thread::scoped( move || {
			loop {
				let (r, n, ov) = get_queued_completion_status( &port );

				if let Some( watch ) = ov {
					// Got an event, handle it:
					if let Err( Error::Io( e ) ) = r {

					}
				} else {
					// No event, handle an instruction:
					match ts.ins_rx.try_recv() {
						Err( mpsc::TryRecvError::Disconnected ) => {
							unreachable!();
						},
						Ok( Close ) => {
							// Close all file handles.
						},
						Ok( Watch( path ) ) => {
							WinFsNotifier::add_watch();
						},
						Ok( Unwatch( path ) ) => {
							WinFsNotifier::rem_watch();
						},
						_ => {}
					}
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
}