use std::io;
pub fn errno() -> i32 {
	io::Error::last_os_error().raw_os_error().unwrap()
}

use std::path::{Path, PathBuf};
use std::convert::AsRef;
pub fn path_buf<P: AsRef<Path> + ?Sized>( path: &P ) -> PathBuf {
	path.as_ref().to_path_buf()
}

use std::hash::Hash;
use std::borrow::Borrow;
use std::collections::HashMap;
pub fn idxmut_map<'a, K, V, Q>( m: &'a mut HashMap<K, V>, i: &Q ) -> &'a mut V
	where
		K: Eq + Hash + Borrow<Q>,
		Q: Eq + Hash {
	m.get_mut( i ).expect( "no entry found for key" )
}

macro_rules! not_implemented {
	() => { return Err( Error::NotImplemented ) }
}

macro_rules! fsn_drop {
	( $concrete:ident ) => {
		impl Drop for $concrete {
			fn drop( &mut self ) {
				self.close().ok().expect( "Failed to stop" );
			}
		}
	}
}