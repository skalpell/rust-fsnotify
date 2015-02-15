extern crate inotify;
use INotify;

use std::path::{AsPath, Path, PathBuf};

use fsnotify::*;

pub struct LinuxFsNotifier<'a> {
	config: Configuration<'a>,
	sender:	EventSender,
}

fsnotify_drop!( LinuxFsNotifier );

impl<'a> FsNotifier<'a> for LinuxFsNotifier<'a> {
	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self> {
		Ok( LinuxFsNotifier {
			config: config,
			sender: sender,
		} )
	}

	fn add( &mut self, path: &AsPath ) -> R {
		not_implemented!();
	}

	fn remove( &mut self, path: &AsPath ) -> R {
		not_implemented!();
	}

	fn stop( &mut self ) -> R {
		not_implemented!();
	}
}