extern crate winapi;
extern crate "kernel32-sys" as win;

use fsnotify::*;

struct WinFsNotifier<'a> {
	config:	Configuration<'a>,
	sender:	EventSender<'a>,
}

fsnotify_drop!( WinFsNotifier );

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
	fn new( sender: EventSender<'a>, config: Configuration<'a> ) -> Self {
		WinFsNotifier {
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
