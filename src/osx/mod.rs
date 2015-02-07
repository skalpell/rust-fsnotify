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