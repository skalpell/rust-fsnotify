use std::ops::Fn;
use std::error::FromError;
use std::sync::mpsc;

use std::path::Path as StdPath;

// @todo, change to std::io once stable.
use std::old_io as io;

//================================================================================
// Error:
//================================================================================

/**
 * `Error` contains all the possible errors for `fsnotify`.
 */
pub enum Error {
	Io( io::IoError ),
	NotifyError( String ),
	PathInvalid,
	NotImplemented,
}

impl FromError<io::IoError> for Error {
	fn from_error( from: io::IoError ) -> Error {
		Error::Io( from )
	}
}

//================================================================================
// Misc typedefs:
//================================================================================

pub type Path = StdPath;

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
pub type RecursionFilter<'a> = Option<&'a (Fn( &Path ) -> bool + 'a)>;

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
 * It configures what operations to subcribe to if applicable for the platform.
 * This lets you avoid tracking events of no interest.
 *
 * It also configures if symlinks should be followed or not (`follow_symlinks`).
 *
 * Lastly, it configures the `RecursionLimit`, and `RecursionFilter`.
 */
pub struct Configuration<'a> {
	subscribe: Operations,
	follow_symlinks: bool,
	recursion_limit: RecursionLimit,
	recursion_filter: RecursionFilter<'a>,
}

impl<'a> Configuration<'a> {
	pub fn is_recursive( &self ) -> bool {
		match self.recursion_limit {
			None => true,
			Some( limit ) => limit > 0
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
pub struct Event<'a> {
	pub path: Option<&'a Path>,
	pub op: NotifyResult<Operations>,
}

/**
 * `EventSender`, a `Sender` for an `Event`.
 */
pub type EventSender<'a> = mpsc::Sender<Event<'a>>;

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
 * Thus, all methods return a `NotifyResult<T> = Result<T, Error>`.
 * All methods but the first, `.new(...)` return `R = NotifyResult<()>`,
 * indicating either success or failure of the operation.
 */
pub trait FsNotifier<'a> : Drop {
	/**
	 * Constructs a `FsNotifier`.
	 *
	 * The sender is an `EventSender` which the notifier will send events through.
	 * The config: `Configuration` contains information about how to handle recursion,
	 * symlinks and what file system operations that should be tracked.
	 *
	 * The returned value is a the notifier, wrapped in a `NotifyResult`.
	 * This is done because there might be I/O failures depending on the system.
	 */
	fn new( sender: EventSender<'a>, config: Configuration<'a> ) -> NotifyResult<Self>;

	/**
	 * Adds a path to track to the notifier.
	 *
	 * Returned is a Result<()>, that indicates either failure or success.
	 */
	fn add( &self, path: &Path ) -> R;

	/**
	 * Tells the notifier to stop tracking a path.
	 *
	 * Returned is a Result<()>, that indicates either failure or success.
	 */
	fn remove( &self, path: &Path ) -> R;

	/**
	 * Tells the notifier to start the tracking.
	 * This operation is blocking, therefore it should be wrapped in a thread.
	 *
	 * Returned is a Result<()>, that indicates either failure or success.
	 */
	fn start( &self ) -> R;

	/**
	 * Tells the notifier to stop the tracking.
	 *
	 * Returned is a Result<()>, that indicates either failure or success.
	 */
	fn stop( &self ) -> R;
}