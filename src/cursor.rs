use boyer_moore_magiclen::BMByte;
use boyer_moore_magiclen::BMByteSearchable;
use std::ops::Range;
use std::slice::Iter;

/// Source stream is not data stream.
/// Data stream is a part of the source stream.   
/// Opernations should be done on the data stream.
pub struct StreamCursor {
    src: Vec<u8>,
    pos_in_data: usize,
    offset: usize,
}

struct SearchableU8<'a> {
    pub value: &'a [u8],
}

impl<'a> From<&'a [u8]> for SearchableU8<'a> {
    fn from(value: &'a [u8]) -> Self {
        SearchableU8 { value }
    }
}

impl<'a> BMByteSearchable for SearchableU8<'a> {
    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(&self.value)
    }

    #[inline]
    fn value_at(&self, index: usize) -> u8 {
        self.value[index]
    }

    #[inline]
    fn iter(&self) -> Iter<u8> {
        <[u8]>::iter(&self.value)
    }
}

impl StreamCursor {
    /// Offset is the start position of the data stream
    pub fn new(src: Vec<u8>, offset: usize) -> Self {
        StreamCursor { src, pos_in_data: 0, offset }
    }

    pub fn mov(&mut self, dist: isize) -> &mut Self {
        let dest = self.pos_in_data as isize + dist;
        if dest < 0 {
            self.pos_in_data = 0;
        } else if dest as usize > self.src.len() {
            self.pos_in_data = self.src.len();
        } else {
            self.pos_in_data = dest as usize;
        }
        self
    }

    /// Start from offset. Real data.
    pub fn data(&self) -> &[u8] {
        &self.src[self.offset..]
    }

    pub fn current(&self) -> &[u8] {
        &self.src[self.pos_in_data + self.offset..]
    }

    /// __pos__ is the position in the data stream, not the whole file stream
    pub fn seek(&mut self, pos: usize) -> &mut Self {
        self.pos_in_data = if pos < self.data().len() { pos } else { self.data().len() };
        self
    }

    /// __pos__ is the position in the data stream, not the whole file stream
    pub fn tell(&self) -> usize {
        self.pos_in_data
    }

    pub fn remain(&self) -> usize {
        self.src.len() - self.pos_in_data - self.offset
    }

    pub fn find(&self, needle: Vec<u8>, range: Range<usize>) -> Option<usize> {
        let bmb = BMByte::from(&needle).unwrap();
        let slice = SearchableU8::from(&self.src[range.start + self.offset..range.end + self.offset]);
        bmb.find_first_in(slice).map(|pos| pos + range.start)
    }

    /// __range__ is the range in the data stream
    pub fn rfind(&self, needle: &Vec<u8>, range: Range<usize>) -> Option<usize> {
        let bmb = BMByte::from(needle).unwrap();
        let slice = SearchableU8::from(&self.src[range.start + self.offset..range.end + self.offset]);
        bmb.rfind_first_in(slice).map(|pos| pos + range.start)
    }
}

impl StreamCursor {
    pub fn peek_u8(&self) -> Option<u8> {
        if self.current().is_empty() {
            return None;
        } else {
            let result = self.src[self.pos_in_data + self.offset];
            return Some(result);
        }
    }

    pub fn get_u8(&mut self) -> Option<u8> {
        let result = self.peek_u8();
        if result.is_some() {
            self.pos_in_data += 1;
        }
        result
    }

    pub fn get_i8(&mut self) -> Option<i8> {
        if self.current().is_empty() {
            return None;
        } else {
            let result = self.src[self.pos_in_data + self.offset] as i8;
            self.pos_in_data += 1;
            return Some(result);
        }
    }

    pub fn peek_u16(&self) -> Option<u16> {
        if self.current().len() < 2 {
            return None;
        } else {
            let result = u16::from_le_bytes(
                self.src[self.pos_in_data + self.offset..self.pos_in_data + 2 + self.offset]
                    .try_into()
                    .expect("Failed to read u16"),
            );
            return Some(result);
        }
    }

    pub fn get_u16(&mut self) -> Option<u16> {
        let result = self.peek_u16();
        if result.is_some() {
            self.pos_in_data += 2;
        }
        result
    }

    pub fn get_i16(&mut self) -> Option<i16> {
        if self.current().len() < 2 {
            return None;
        } else {
            let result = i16::from_le_bytes(
                self.src[self.pos_in_data + self.offset..self.pos_in_data + 2 + self.offset]
                    .try_into()
                    .expect("Failed to read i16"),
            );
            self.pos_in_data += 2;
            return Some(result);
        }
    }

    pub fn get_i32(&mut self) -> Option<i32> {
        if self.current().len() < 4 {
            return None;
        } else {
            let result = i32::from_le_bytes(
                self.data()[self.pos_in_data..self.pos_in_data + 4].try_into().expect("Failed to read i32"),
            );
            self.pos_in_data += 4;
            return Some(result);
        }
    }

    pub fn peek_u32(&self) -> Option<u32> {
        if self.current().len() < 4 {
            return None;
        } else {
            let result = u32::from_le_bytes(
                self.data()[self.pos_in_data..self.pos_in_data + 4].try_into().expect("Failed to read u32"),
            );
            return Some(result);
        }
    }

    pub fn get_u32(&mut self) -> Option<u32> {
        let result = self.peek_u32();
        if result.is_some() {
            self.pos_in_data += 4;
        }
        result
    }

    pub fn peek_i32(&self) -> Option<i32> {
        if self.current().len() < 4 {
            return None;
        } else {
            let result = i32::from_le_bytes(
                self.data()[self.pos_in_data..self.pos_in_data + 4].try_into().expect("Failed to read i32"),
            );
            return Some(result);
        }
    }

    pub fn peek_f32(&self) -> Option<f32> {
        if self.current().len() < 4 {
            return None;
        } else {
            let result = f32::from_le_bytes(
                self.data()[self.pos_in_data..self.pos_in_data + 4].try_into().expect("Failed to read f32"),
            );
            return Some(result);
        }
    }

    pub fn get_f32(&mut self) -> Option<f32> {
        let result = self.peek_f32();
        if result.is_some() {
            self.pos_in_data += 4;
        }
        result
    }

    pub fn get_bool(&mut self, bytes: u8) -> Option<bool> {
        // if next byets bytes are all 0, return false, otherwise true
        let mut result = false;
        for i in 0..bytes {
            if self.data()[self.pos_in_data + i as usize] != 0 {
                result = true;
                break;
            }
        }
        self.pos_in_data += bytes as usize;
        Some(result)
    }

    pub fn extract_str_l32(&mut self) -> Option<Vec<u8>> {
        let str_len = self.get_i32()? as usize;
        if str_len == 0 {
            return None;
        }
        let raw_str: Vec<u8>;
        if self.current()[str_len - 1] == 0 {
            raw_str = (&self.current()[0..str_len - 1]).to_vec();
        } else {
            raw_str = (&self.current()[0..str_len]).to_vec();
        };
        self.mov(str_len as isize);
        Some(raw_str)
    }

    pub fn extract_str_l16(&mut self) -> Option<Vec<u8>> {
        let str_len = self.get_u16()? as usize;
        if str_len == 0 {
            return None;
        }
        let raw_str: Vec<u8>;
        if self.current()[str_len - 1] == 0 {
            raw_str = (&self.current()[0..str_len - 1]).to_vec();
        } else {
            raw_str = (&self.current()[0..str_len]).to_vec();
        };
        self.mov(str_len as isize);
        Some(raw_str)
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub fn print_hex(&self, len: usize) {
        let mut i = 0;
        println!("{} bytes from {}:", len, self.pos_in_data);
        while i < len {
            print!("{:02x} ", self.data()[self.pos_in_data + i]);
            i += 1;
        }
        println!();
    }
}
