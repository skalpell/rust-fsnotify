extern crate fsevent;
extern crate "fsevent-sys" as fsvent_fys;
use fsevent;

use fsnotify::*;

struct OsxFsNotifier<'a> {
	config: Configuration<'a>,
	sender:	EventSender<'a>,
}

fsnotify_drop!( OsxFsNotifier );

impl<'a> FsNotifier<'a> for OsxFsNotifier<'a> {
	fn new( sender: EventSender, config: Configuration<'a> ) -> Self {
		return OsxFsNotifier {
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