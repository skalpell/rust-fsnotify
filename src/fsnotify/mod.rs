use std::default::Default;
use std::error::FromError;
use std::ops::Fn;
use std::io;
use std::any::Any;
use std::marker::Sized;
use std::path::{
	PathBuf,
	AsPath,
};
use std::sync::{
	mpsc,
	PoisonError,
	RwLockReadGuard,
	RwLockWriteGuard,
};

//================================================================================
// Error handling:
//================================================================================

/**
 * `Error` contains all the possible errors for `fsnotify`.
 */
pub enum Error {
	Notify( String ),
	Io( io::Error ),
	Sending,
	Closed,
	LockWrite,
	LockRead,
	ThreadPanic,
	PathInvalid,
	NotImplemented,
}

impl FromError<io::Error> for Error {
	fn from_error( from: io::Error ) -> Error {
		Error::Io( from )
	}
}

impl<T> FromError<mpsc::SendError<T>> for Error {
	fn from_error( from: mpsc::SendError<T> ) -> Error {
		Error::Sending
	}
}

impl FromError<Box<Any + Send>> for Error {
	fn from_error( from: Box<Any + Send> ) -> Error {
		Error::ThreadPanic
	}
}

impl<'a, T> FromError<PoisonError<RwLockReadGuard<'a, T>>> for Error {
	fn from_error( from: PoisonError<RwLockReadGuard<T>> ) -> Error {
		Error::LockRead
	}
}

impl<'a, T> FromError<PoisonError<RwLockWriteGuard<'a, T>>> for Error {
	fn from_error( from: PoisonError<RwLockWriteGuard<T>> ) -> Error {
		Error::LockWrite
	}
}

//================================================================================
// Misc typedefs:
//================================================================================

/**
 * The `Result` of an operation, with either `T` (success), `Error` (failure).
 */
pub type NotifyResult<T> = Result<T, Error>;

/**
 * The `Result` of an `Event`, either being the `Event`, or an `Error`.
 */
pub type EventResult = NotifyResult<Event>;

/**
 * `R = NotifyResult<()`
 * Indicates either success or failure of an operation.
 */
pub type R = NotifyResult<()>;

//================================================================================
// Configuration:
//================================================================================

type RFCallback = Fn( &AsPath ) -> bool + Send;

/**
 * `RecursionFilter`, a predicate function that, when recursion is enabled,
 * tells the `FsNotifier` if it should subscribe to a sub-directory.
 *
 * If the value returned is true, it will subscribe, otherwise, it will not.
 *
 * It is optional to provide such a filter.
 */
pub type RecursionFilter = Option<Box<RFCallback>>;

/**
 * `RecursionLimit` denoted what the maximum recursion depth is starting
 * from a tree point that was explicitly added to the watch list.
 *
 * `None` means that there is no limit, it will do recursion forever.
 * `Some(0)` means that no recursion will happen at all.
 */
pub type RecursionLimit = Option<usize>;

/**
 * `Configuration` provides configuration for a `FsNotifier`.
 *
 * The following configurations are available:
 * + `subscribe`: the operations to subcribe to if applicable for the platform.
 *    	This lets you avoid tracking events of no interest.
 *    	Default is to subscribe to everything.
 * + `follow_symlinks`: should symlinks be followed?
 * 		Default is yes.
 * + `recursion_limit`: see `RecursionLimit`.
 * 		Default is 0.
 * + `recursion_filter`: see `RecursionFilter`.
 * 		Default is to not filter anything.
 */
pub struct Configuration {
	subscribe:			Operations,
	follow_symlinks:	bool,
	recursion_limit:	RecursionLimit,
	recursion_filter:	RecursionFilter,
}

impl Configuration {
	pub fn is_recursive( &self ) -> bool {
		match self.recursion_limit {
			None => true,
			Some( limit ) => limit > 0
		}
	}
}

impl Default for Configuration {
	fn default() -> Configuration {
		Configuration {
			subscribe:			Operations::all(),
			follow_symlinks:	true,
			recursion_limit:	Some( 0 ),
			recursion_filter:	None,
		}
	}
}

//================================================================================
// Events:
//================================================================================

mod operations;
use self::operations::Operations;

/**
 * Event:s are passed to the `EventSender`.
 * It has information about the path that the operations that happened on it.
 */
pub struct Event {
	pub path:	Option<PathBuf>,
	pub op:		Operations,
}

impl Event {
	fn new( path: Option<PathBuf>, op: Operations ) -> Self {
		Event { path: path, op: op }
	}
}

/**
 * `EventSender`, a `Sender` for an `Event`.
 */
pub type EventSender = mpsc::Sender<EventResult>;

//================================================================================
// Notifier trait:
//================================================================================

/**
 * FsNotifier, a trait, is the generic interface for
 * subscribing to file system change events.
 *
 * It provides 4 basic operations:
 *  + `::new()`
 * 	+ `.watch( &AsPath )`
 * 	+ `.unwatch( &AsPath )`
 * 	+ `.close()`
 *
 * As this trait deals with I/O interaction with the operating system, things can fail.
 * Thus, all methods except for `FsNotifier::new` return a `R = Result<(), Error>`
 * indicating either success or failure.
 */
pub trait FsNotifier : Drop {
	/**
	 * Constructs a `FsNotifier`.
	 *
	 * The sender is an `EventSender` which the notifier will send events through.
	 * The config: `Configuration` contains configuration information.
	 *
	 * This spawns a new thread that the notifier runs in.
	 * The thread should not be considerered as detached.
	 */
	fn new( sender: EventSender, config: Configuration ) -> NotifyResult<Self>;

	/**
	 * Adds a path to track to the notifier.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn watch<P: AsPath + ?Sized>( &mut self, path: &P ) -> R;

	/**
	 * Tells the notifier to stop tracking a path.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn unwatch<P: AsPath + ?Sized>( &mut self, path: &P ) -> R;

	/**
	 * Tells the notifier to stop the tracking.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn close( &mut self ) -> R;

	/**
	 * Indicates whether or not the notifier is `closed` or if it is `running`.
	 *
	 * `true` if it is closed.
	 */
	fn is_closed( &self ) -> NotifyResult<bool>;
}