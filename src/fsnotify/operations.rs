bitflags! {
	flags Operations: u32 {
		const CREATE		= 0x0001,
		const DELETE		= 0x0002,
		const MOVE			= 0x0004,
		const ACCESS		= 0x0008,
		const ATTRIB		= 0x0010,
		const MODIFY		= 0x0020,
		const OPEN			= 0x0040,
		const CLOSE_WRITE	= 0x0080,
		const CLOSE_NOWRITE = 0x0100,
		const UNMOUNT		= 0x0240,
		const MOUNT			= 0x0400,
		const IS_DIR		= 0x0800,
		const OVERFLOW		= 0x1000,
	}
}