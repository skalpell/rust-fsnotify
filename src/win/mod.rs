extern crate winapi;

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

use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::collections::hash_map::{HashMap, Entry};
use std::option::Option;
use std::marker::Send;
use std::mem;

use fsnotify::*;
use helpers::*;

mod ffi;
use self::ffi::*;

//================================================================================
// Enum for sending Close/Watch/Unwatch instructions:
//================================================================================

enum Instruction {
	Close,
	Watch(	 PathBuf ),
	Unwatch( PathBuf ),
}

type InsTx = mpsc::Sender<(Instruction, ReplyTx)>;
type InsRx = mpsc::Receiver<(Instruction, ReplyTx)>;

type ReplyTx = mpsc::Sender<R>;
type ReplyRx = mpsc::Receiver<R>;

//================================================================================
// The struct:
//================================================================================

pub struct WinFsNotifier {
	/// Indicates if the notifier is closed or not.
	is_closed:	bool,

	/// Join guard, this is held until Drop.
	join_guard:	Option<JoinHandle>,

	/// Close/Watch/Unwatch instruction TX.
	ins_tx:		InsTx,

	/// Handle to completion port:
	port:		Arc<HANDLE>,
}

fsn_drop!( WinFsNotifier );

//================================================================================
// Buffer trait:
//================================================================================

/**
 * `BufferContainer` is used to allocate the buffers for ReadDirectoryChanges.
 * Lets the user use a larger buffer, and heap allocate instead of on the stack.
 */
trait BufferContainer: AsRef<[u8]> + AsMut<[u8]> {
	/// Creates a container.
	fn new() -> Self;
}

/**
 * Creates a BufferContainer that allocates on the stack.
 * Usage: `stack_buffer_container!( MyBufferContainer, 2048 );`
 */
macro_rules! stack_buffer_container {
	( $clazz: ident, $size: expr ) => {
		struct $clazz( [u8; $size] );

		impl BufferContainer for $clazz {
			fn new() -> Self { $clazz( unsafe { mem::uninitialized() } ) }
		}

		impl AsMut<[u8]> for $clazz {
			fn as_mut( &mut self ) -> &mut [u8] { &mut self.0 }
		}

		impl AsRef<[u8]> for $clazz {
			fn as_ref( &self ) -> &[u8] { &self.0 }
		}
	}
}

// Define our standard StackBufferContainer.
stack_buffer_container!( StackBufferContainer, 4096 );

//================================================================================
// Implementation:
//================================================================================

struct Watch<B: BufferContainer = StackBufferContainer> {
	ov:		OVERLAPPED,
	inode:	INode,
	path:	PathBuf,
	buffer:	B,
}
unsafe impl<B: BufferContainer> Send for Watch<B> {}

type IndexMap<B: BufferContainer> = HashMap<u64, Watch<B>>;
type WatchMap<B: BufferContainer> = HashMap<u32, IndexMap<B>>;

struct ThreadState<B: BufferContainer = StackBufferContainer> {
	/// The configuration.
	config:		Configuration,

	/// Event notification TX.
	event_tx:	EventSender,

	/// Close/Watch/Unwatch instruction RX.
	ins_rx:		InsRx,

	/// Maps Volume -> INode -> Watch.
	watches:	WatchMap<B>,
}

impl FsNotifier for WinFsNotifier {
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
			watches:	HashMap::new(),
		} );

		Ok( n )
	}

	fn watch<P: AsRef<Path> + ?Sized>( &mut self, path: &P ) -> R {
		// Send watch signal.
		try!( self.enforce_open() );
		self.instruct( Instruction::Watch( path_buf( path ) ) )
	}

	fn unwatch<P: AsRef<Path> + ?Sized>( &mut self, path: &P ) -> R {
		// Send unwatch signal.
		try!( self.enforce_open() );
		self.instruct( Instruction::Unwatch( path_buf( path ) ) )
	}

	fn close( &mut self ) -> R {
		try!( self.enforce_open() );

		// Consider us closed, don't want this called again.
		self.is_closed = true;

		// Send Close signal.
		try!( self.instruct( Instruction::Close ) );

		// Block/Join until thread terminates.
		if let Some( jg ) = self.join_guard.take() {
			Ok( try!( jg.join() ) )
		} else {
			unreachable!();
		}
	}

	fn is_closed( &self ) -> NotifyResult<bool> {
		Ok( self.is_closed )
	}
}

impl WinFsNotifier {
	/**
	 * Sends instruction to thread loop.
	 * Called in client thread.
	 */
	fn instruct( &self, ins: Instruction ) -> R {
		let (tx, rx) = mpsc::channel();
		try!( self.ins_tx.send( (ins, tx) ) );
		try!( post_queued_completion_status( *self.port ) );
		try!( rx.recv() )
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

	fn run( &mut self, mut ts: ThreadState ) {
		// Clone Arc:s.
		let (is_closed, port) = (
			self.is_closed.clone(),
			self.port.clone()
		);

		// Run notifier in thread.
		self.join_guard = Some( thread::spawn( move || {
			loop {
				let (r, n, ov) = get_queued_completion_status( *port );
				if ov.is_null() {
					ts.handle_instruction( &port )
				} else {
					let watch: &Watch = unsafe { mem::transmute( ov ) };

					// Got an event, handle it:
					if let Err( Error::Io( e ) ) = r {
						match errno() {
							ERROR_MORE_DATA => {
								/*
								 * The i/o succeeded but the buffer is full.
								 * In theory we should be building up a full packet.
								 * In practice we can get away with just carrying on.
								 */
						//		n = &watch.buffer.len();
							},
							ERROR_ACCESS_DENIED => {
								// Watched directory was probably removed.
								//w.sendEvent(watch.path, watch.mask&sys_FS_DELETE_SELF)
						//		ts.delete_watch( watch );
						//		ts.start_read( watch );
								continue;
							},
							ERROR_OPERATION_ABORTED => {
								// CancelIo was called on this handle.
								continue;
							},
							_ => {
								ts.event_error( Error::Io( e ) );
								continue;
							}
						}
					}

					let mut offset: u32;
					loop {
						if n == 0 {
				// w.internalEvent <- &FileEvent{mask: sys_FS_Q_OVERFLOW}
				// w.Error <- errors.New("short read in readEvents()")
							break;
						}

						// Point "raw" to the event in the buffer.

					}
				}

				// Are we using recursion?
				//let recurse = self.config.is_recursive() as BOOL;
			}
		} ) );
	}
}

impl<B> ThreadState<B>
where B: BufferContainer {
	fn event( &self, er: EventResult ) {
		self.event_tx.send( er ).unwrap();
	}

	fn event_error( &self, error: Error ) {
		self.event( Err( error ) )
	}

	fn handle_instruction( &mut self, port: &HANDLE ) {
		// No event, handle an instruction:
		use self::Instruction::*;
		match self.ins_rx.try_recv() {
			Err( mpsc::TryRecvError::Disconnected ) => {
				unreachable!();
			},
			Ok( (Close, tx) ) => {
				// Close all file handles.
				tx.send( self.close( *port ) ).unwrap()
			},
			Ok( (Watch( path ), tx) ) => {
				tx.send( self.add_watch( *port, path ) ).unwrap();
			},
			Ok( (Unwatch( path ), tx) ) => {
				tx.send( self.rem_watch( *port, path ) ).unwrap();
			},
			_ => {}
		}
	}

	fn close( &mut self, port: HANDLE ) -> R {
		for vol in self.watches.values() {
			for watch in vol.values() {
			//	w.deleteWatch(watch)
			//	w.startRead(watch)
			}
		}

		close_handle( port )
	}

	fn add_watch( &mut self, port: HANDLE, path: PathBuf ) -> R {
		// Directory stuff:
		let dir = path;

		// Get inode.
		let inode = try!( file_inode( &dir ) );

		// Got watch for it? Add if not.
		let mut watch = match self.watch_entry( &inode ) {
			Entry::Occupied( e ) => {
				// We have it, don't want duplicate.
				try!( close_handle( inode.handle ) );
				e.into_mut()
			},
			Entry::Vacant( e ) => {
				if let Err( err ) = create_io_completion_port( inode.handle, port ) {
					// Houston, we have a problem.
					try!( close_handle( inode.handle ) );
					return Err( err );
				} else {
					e.insert( Watch {
						inode:	inode,
						path:	dir,
						ov:		make_overlapped(),
						buffer:	BufferContainer::new(),
					} )
				}
			}
		};

	//	set_if( index, || {exists_do}, ||{not_do}

		Ok( () )
	}

	fn rem_watch( &mut self, port: HANDLE, path: PathBuf ) -> R {
		// Directory stuff:
		let dir = path;

		// Get inode, do we already a watch for it?
		let inode = try!( file_inode( &dir ) );
		if let Entry::Occupied( _ ) = self.watch_entry( &inode ) {
			Ok( () )
		} else {
			Err( Error::PathNotWatched )
		}
	}

	fn start_read( &mut self, watch: &mut Watch ) -> R {
		if let Err( e ) = cancel_io( watch.inode.handle ) {
			self.delete_watch( watch );
			return Err( e );
		}

		// Fiddle with masks:
		let mask: u32 = 0;

		if mask == 0 {
			// Ordered to unwatch watch.
			idxmut_map( &mut self.watches, &watch.inode.volume ).remove( &watch.inode.index );
			return close_handle( watch.inode.handle );
		} else if let Err( e ) = read_directory_changes( watch.inode.handle, watch.buffer.as_mut(), false, mask, &mut watch.ov ) {
			let r: R;

			if errno() == ERROR_ACCESS_DENIED { // && watch.mask&provisional == 0 {
				// Watched directory was probably removed
				/*
				if w.sendEvent(watch.path, watch.mask&sys_FS_DELETE_SELF) {
					if watch.mask&sys_FS_ONESHOT != 0 {
						watch.mask = 0
					}
				}
				*/
				r = Ok( () );
			} else {
				r = Err( e );
			}

			self.delete_watch( watch );
			try!( self.start_read( watch ) );

			return r;
		} else {
			return Ok( () );
		}
	}

	fn delete_watch( &mut self, watch: &Watch ) {

	}

	fn watch_entry( &mut self, inode: &INode ) -> Entry<u64, Watch<B>> {
		self.watches
			.entry( inode.volume ).or_insert_with( || HashMap::new() )
			.entry( inode.index )
	}
}