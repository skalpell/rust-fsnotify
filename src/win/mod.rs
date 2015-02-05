extern crate winapi;
extern crate "kernel32-sys" as kernel;

use fsnotify::*;

struct WinFsNotifier<'a> {
	config: Configuration<'a>,
}

fsnotify_drop!( WinFsNotifier );

impl<'a> FsNotifier<'a> for WinFsNotifier<'a> {
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