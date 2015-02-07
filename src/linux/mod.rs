extern crate inotify;
use INotify;

use fsnotify::*;

struct LinuxFsNotifier<'a> {
	config: Configuration<'a>,
	sender:	EventSender<'a>,
}

fsnotify_drop!( LinuxFsNotifier );

impl<'a> FsNotifier<'a> for LinuxFsNotifier<'a> {
	fn new( sender: EventSender, config: Configuration<'a> ) -> Self {
		return LinuxFsNotifier {
			config: config,
			sender: sender,
		}
	}

	fn add( &mut self, path: &Path ) -> R {
		not_implemented!();
	}

	fn remove( &mut self, path: &Path ) -> R {
		not_implemented!();
	}

	fn start( &mut self ) -> R {
		not_implemented!();
	}

	fn stop( &mut self ) -> R {
		not_implemented!();
	}
}