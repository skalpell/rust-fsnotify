use std::path::{AsPath, PathBuf};

pub fn path_buf<P: AsPath + ?Sized>( path: &P ) -> PathBuf {
	path.as_path().to_path_buf()
}

macro_rules! not_implemented {
	() => { return Err( Error::NotImplemented ) }
}

macro_rules! fsn_drop {
	( $concrete:ident ) => {
		#[unsafe_destructor]
		impl<'a> Drop for $concrete<'a> {
			fn drop( &mut self ) {
				self.close().ok().expect( "Failed to stop" );
			}
		}
	}
}