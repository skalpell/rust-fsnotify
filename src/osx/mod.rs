extern crate fsevent;
extern crate fsevent_sys;
use fsevent;

use std::path::{AsPath, Path, PathBuf};

use fsnotify::*;

pub struct OsxFsNotifier<'a> {
	config: Configuration<'a>,
	sender:	EventSender,
}

fsnotify_drop!( OsxFsNotifier );

impl<'a> FsNotifier<'a> for OsxFsNotifier<'a> {
	fn new( sender: EventSender, config: Configuration<'a> ) -> NotifyResult<Self> {
		Ok( OsxFsNotifier {
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