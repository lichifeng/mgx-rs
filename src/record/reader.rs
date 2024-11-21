use crate::record::Record;

impl Record {
    pub fn hf32(&mut self) -> f32 {
        let src = self.get_header();
        let f = f32::from_le_bytes(src[0..4].try_into().expect("Failed to read f32"));
        self.debug.currentpos_header += 4;
        f
    }

    pub fn hu32(&mut self) -> u32 {
        let src = self.get_header();
        let u = u32::from_le_bytes(src[0..4].try_into().expect("Failed to read u32"));
        self.debug.currentpos_header += 4;
        u
    }

    pub fn hu16(&mut self) -> u16 {
        let src = self.get_header();
        let u = u16::from_le_bytes(src[0..2].try_into().expect("Failed to read u16"));
        self.debug.currentpos_header += 2;
        u
    }

    pub fn hu8(&mut self) -> u8 {
        let src = self.get_header();
        let u = src[0];
        self.debug.currentpos_header += 1;
        u
    }

    pub fn hi32(&mut self) -> i32 {
        let src = self.get_header();
        let i = i32::from_le_bytes(src[0..4].try_into().expect("Failed to read i32"));
        self.debug.currentpos_header += 4;
        i
    }
}
