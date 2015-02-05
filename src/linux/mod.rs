extern crate inotify;
use INotify;

use fsnotify::*;

struct LinuxFsNotifier<'a> {
	config: Configuration<'a>,
	sender:	EventSender<'a>,
}

fsnotify_drop!( LinuxFsNotifier );

impl<'a> FsNotifier<'a> for LinuxFsNotifier<'a> {
	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self> {
		not_implemented!();
	}

	fn add( &self, path: &Path ) -> R {
		not_implemented!();
	}

	fn remove( &self, path: &Path ) -> R {
		not_implemented!();
	}

	fn start( &self ) -> R {
		not_implemented!();
	}

	fn stop( &self ) -> R {
		not_implemented!();
	}
}