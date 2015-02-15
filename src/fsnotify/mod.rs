use std::default::Default;
use std::error::FromError;
use std::ops::Fn;
use std::sync::{
	mpsc,
	PoisonError,
	RwLockReadGuard,
	RwLockWriteGuard,
};
use std::path::{
	Path,
	PathBuf,
};

// @TODO, change to std::io once stable.
use std::old_io as io;

//================================================================================
// Error:
//================================================================================

/**
 * `Error` contains all the possible errors for `fsnotify`.
 */
pub enum Error {
	NotifyError( String ),
	Io( io::IoError ),
	LockWriteError,
	LockReadError,
	PathInvalid,
	NotImplemented,
}

impl<> FromError<io::IoError> for Error {
	fn from_error( from: io::IoError ) -> Error {
		Error::Io( from )
	}
}

impl<'a, T> FromError<PoisonError<RwLockReadGuard<'a, T>>> for Error {
	fn from_error( from: PoisonError<RwLockReadGuard<T>> ) -> Error {
		Error::LockWriteError
	}
}

impl<'a, T> FromError<PoisonError<RwLockWriteGuard<'a, T>>> for Error {
	fn from_error( from: PoisonError<RwLockWriteGuard<T>> ) -> Error {
		Error::LockWriteError
	}
}

//================================================================================
// Misc typedefs:
//================================================================================

pub type FilePath<'a> = &'a Path;

/**
 * The `Result` of an operation, with either `T` (success), `Error` (failure).
 */
pub type NotifyResult<T> = Result<T, Error>;

/**
 * `R = NotifyResult<()`
 * Indicates either success or failure of an operation.
 */
pub type R = NotifyResult<()>;

//================================================================================
// Configuration:
//================================================================================

/**
 * `RecursionFilter`, a predicate function that, when recursion is enabled,
 * tells the `FsNotifier` if it should subscribe to a sub-directory.
 *
 * If the value returned is true, it will subscribe, otherwise, it will not.
 */
pub type RecursionFilter<'a> = Option<&'a (Fn( FilePath ) -> bool + 'a)>;

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
pub struct Configuration<'a> {
	subscribe:			Operations,
	follow_symlinks:	bool,
	recursion_limit:	RecursionLimit,
	recursion_filter:	RecursionFilter<'a>,
}

impl<'a> Configuration<'a> {
	pub fn is_recursive( &self ) -> bool {
		match self.recursion_limit {
			None => true,
			Some( limit ) => limit > 0
		}
	}
}

impl<'a> Default for Configuration<'a> {
	fn default() -> Configuration<'a> {
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
	pub path: Option<PathBuf>,
	pub op: NotifyResult<Operations>,
}

/**
 * `EventSender`, a `Sender` for an `Event`.
 */
pub type EventSender = mpsc::Sender<Event>;

//================================================================================
// Notifier trait:
//================================================================================

/**
 * FsNotifier, a trait, is the generic interface for
 * subscribing to file system change events.
 *
 * It provides 4 basic operations:
 * 	+ `.add( &Path )`
 * 	+ `.remove( &Path )`
 * 	+ `.start()`
 * 	+ `.stop()`
 *
 * As this trait deals with I/O interaction with the operating system, things can fail.
 * Thus, all methods return a `R = Result<(), Error>` indicating either success or failure.
 */
pub trait FsNotifier<'a> : Drop {
	/**
	 * Constructs a `FsNotifier`.
	 *
	 * The sender is an `EventSender` which the notifier will send events through.
	 * The config: `Configuration` contains configuration information.
	 *
	 * This spawns a new thread that the notifier runs in.
	 */
	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self>;

	/**
	 * Adds a path to track to the notifier.
	 * This can be done after calling `start()`.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn watch( &mut self, path: FilePath ) -> R;

	/**
	 * Tells the notifier to stop tracking a path.
	 * This can be done after calling `start()`.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn unwatch( &mut self, path: FilePath ) -> R;

	/**
	 * Tells the notifier to stop the tracking.
	 *
	 * Returned is a `R`, that indicates either failure or success.
	 */
	fn close( &mut self ) -> R;
}