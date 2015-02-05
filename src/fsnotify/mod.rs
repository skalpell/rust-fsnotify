use std::old_io as io;
use std::sync::mpsc;

use std::path::Path as StdPath;

//================================================================================
// Misc typedefs:
//================================================================================
pub type Path = StdPath;
pub type NotifyResult<T> = io::IoResult<T>;
pub type R = NotifyResult<()>;

//================================================================================
// Recursion:
//================================================================================
pub type RecursionLimit = Option<usize>;
pub struct RecursionConfig<'a> {
	limit: RecursionLimit,
	filter: Fn( &Path ) -> bool + 'a
}

//================================================================================
// Events:
//================================================================================

mod operations;
use self::operations::Operations;

pub struct Event {
	pub path: Option<Path>,
	pub op: NotifyResult<Operations>,
}

pub type EventSender = mpsc::Sender<Event>;

//================================================================================
// Notifier trait:
//================================================================================
pub trait FsNotifier : Drop {
	fn new( sender: EventSender, recursion_limit: RecursionConfig, follow_symlinks: bool ) -> NotifyResult<Self>;

	fn add( &self, path: &Path, subscribe: Operations ) -> R;

	fn remove( &self, path: &Path ) -> R;

	fn start( &self ) -> R;

	fn stop( &self ) -> R;
}