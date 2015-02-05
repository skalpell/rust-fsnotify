use fsnotify::*;

use std::ffi::{
	CString,
};

/**
 * `path_to_cstr` converts from a `Path` to a `Result` wrapped `CString`.
 * It might fail due to unicode problems.
 */
pub fn path_to_cstr( path: &Path ) -> NotifyResult<CString> {
	match path.to_str() {
		None => Err( Error::PathInvalid ),
		Some( conv ) => Ok( CString::from_slice( conv.as_bytes() ) )
	}
}
