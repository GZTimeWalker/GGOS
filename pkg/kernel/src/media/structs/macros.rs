macro_rules! define_field {
    (u8, $offset:expr, $name:ident) => {
        /// Get the value from the $name field
        pub fn $name(&self) -> u8 {
            self.data[$offset]
        }
    };

    (u16, $offset:expr, $name:ident) => {
        /// Get the value from the $name field
        pub fn $name(&self) -> u16 {
            u16::from_le_bytes(&self.data[$offset..$offset+2])
        }
    };

    (u32, $offset:expr, $name:ident) => {
        /// Get the $name field
        pub fn $name(&self) -> u32 {
            u32::from_le_bytes(&self.data[$offset..$offset+4])
        }
    };
}
