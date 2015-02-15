macro_rules! not_implemented {
	() => { return Err( Error::NotImplemented ) }
}

macro_rules! fsn_run {
	( $stuff: expr ) => {
		{
			let mut n = $stuff;
			n.run();
			Ok( n )
		}
	}
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

macro_rules! fsn_close {
	() => {
		fn close( &mut self ) -> R {
			(*try!( (*self.open).write() )) = false;
			Ok(())
		}
	}
}