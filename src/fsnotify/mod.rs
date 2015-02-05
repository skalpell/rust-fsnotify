use std::old_io as io;
use std::sync::mpsc;

use std::ops::Fn;
use std::path::Path as StdPath;

//================================================================================
// Misc typedefs:
//================================================================================

pub type Path = StdPath;
pub type NotifyResult<T> = io::IoResult<T>;
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

pub struct Event {
	pub path: Option<Path>,
	pub op: NotifyResult<Operations>,
}

pub type EventSender = mpsc::Sender<Event>;

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