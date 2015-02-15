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

macro_rules! fsn_fromerror {
	( $_clazz: ident, $_to: path, $_from: ty ) => {
		impl<> FromError<$_from> for $_clazz {
			fn from_error( from: $_from ) -> $_clazz {
				$_to( from )
			}
		}
	};
	( $_clazz: ident, $_to: path, $_from: ty, $( $_lt: ident ),+ ) => {
		impl<$( $_lt ),+> FromError<$_from<$( $_lt ),+>> for $_clazz {
			fn from_error( from: $_from ) -> $_clazz {
				$_to
			}
		}
	};
}