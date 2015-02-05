use std::ops::Fn;
use std::error::FromError;
use std::sync::mpsc;

use std::path::Path as StdPath;

// @todo, change to std::io once stable.
use std::old_io as io;

//================================================================================
// Error:
//================================================================================

pub enum Error {
	Io( io::IoError ),
	NotifyError( String ),
	PathNotFound,
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
pub type NotifyResult<T> = Result<T, Error>;
pub type R = NotifyResult<()>;

//================================================================================
// Configuration:
//================================================================================

pub type RecursionFilter<'a> = Option<&'a (Fn( &Path ) -> bool + 'a)>;
pub type RecursionLimit = Option<usize>;
pub struct Configuration<'a> {
	subscribe: Operations,
	follow_symlinks: bool,
	recursion_limit: RecursionLimit,
	recursion_filter: RecursionFilter<'a>,
}

//================================================================================
// Events:
//================================================================================

mod operations;
use self::operations::Operations;

pub struct Event<'a> {
	pub path: Option<&'a Path>,
	pub op: NotifyResult<Operations>,
}

pub type EventSender<'a> = mpsc::Sender<Event<'a>>;

//================================================================================
// Notifier trait:
//================================================================================

pub trait FsNotifier<'a> : Drop {
	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self>;

	fn add( &self, path: &Path ) -> R;

	fn remove( &self, path: &Path ) -> R;

	fn start( &self ) -> R;

	fn stop( &self ) -> R;
}