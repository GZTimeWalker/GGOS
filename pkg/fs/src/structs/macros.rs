macro_rules! define_field {
    (u8, $offset:expr, $name:ident) => {
        /// Get the value from the $name field
        pub fn $name(&self) -> u8 {
            self.data.get($offset).unwrap_or(&0).clone()
        }
    };

    (u16, $offset:expr, $name:ident) => {
        /// Get the value from the $name field
        pub fn $name(&self) -> u16 {
            u16::from_le_bytes(self.data[$offset..$offset + 2].try_into().unwrap_or([0; 2]))
        }
    };

    (u32, $offset:expr, $name:ident) => {
        /// Get the $name field
        pub fn $name(&self) -> u32 {
            u32::from_le_bytes(self.data[$offset..$offset + 4].try_into().unwrap_or([0; 4]))
        }
    };

    ([u8; $len:expr], $offset:expr, $name:ident) => {
        /// Get the value from the $name field
        pub fn $name(&self) -> &[u8; $len] {
            (&self.data[$offset..$offset + $len])
                .try_into()
                .unwrap_or(&[0; $len])
        }

        paste::item! {
            /// Get the &str from the $name field
            pub fn [<$name _str>](&self) -> &str {
                core::str::from_utf8(&self.data[$offset..$offset+$len]).unwrap_or("")
            }
        }
    };
}
