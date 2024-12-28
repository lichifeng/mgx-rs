use boyer_moore_magiclen::BMByte;
use boyer_moore_magiclen::BMByteSearchable;
use std::ops::Range;
use std::slice::Iter;

/// Source stream is not data stream.
/// Data stream is a part of the source stream.   
/// Opernations should be done on the data stream.
pub struct StreamCursor {
    pub src: Vec<u8>,
    pub pos_in_data: usize,
    pub offset: usize,
}

// Due to limitation of the boyer_moore_magiclen crate, we need to implement the BMByteSearchable trait for u8 slice.
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
        } else if dest as usize > self.data().len() {
            self.pos_in_data = self.data().len();
        } else {
            self.pos_in_data = dest as usize;
        }
        self
    }

    /// Start from offset. Real data I need.
    pub fn data(&self) -> &[u8] {
        &self.src[self.offset..]
    }

    pub fn current(&self) -> &[u8] {
        &self.src[self.pos_in_data + self.offset..]
    }

    pub fn seek(&mut self, pos_in_data: usize) -> &mut Self {
        self.pos_in_data = if pos_in_data < self.data().len() { pos_in_data } else { self.data().len() };
        self
    }

    /// Return the position in the actual data stream
    pub fn tell(&self) -> usize {
        self.pos_in_data
    }

    pub fn remain(&self) -> usize {
        self.src.len() - self.pos_in_data - self.offset
    }

    /// `range` is the range in the actual data stream
    pub fn find(&self, needle: Vec<u8>, range: Range<usize>) -> Option<usize> {
        if let Some(bmb) = BMByte::from(&needle) {
            let slice = SearchableU8::from(&self.src[range.start + self.offset..range.end + self.offset]);
            return bmb.find_first_in(slice).map(|pos| pos + range.start);
        }
        None
    }

    /// `range` is the range in the actual data stream
    pub fn rfind(&self, needle: &Vec<u8>, range: Range<usize>) -> Option<usize> {
        if let Some(bmb) = BMByte::from(needle) {
            let slice = SearchableU8::from(&self.src[range.start + self.offset..range.end + self.offset]);
            return bmb.rfind_first_in(slice).map(|pos| pos + range.start);
        }
        None
    }
}

impl StreamCursor {
    pub fn peek_u8(&self) -> Option<u8> {
        if self.current().is_empty() {
            None
        } else {
            let result = self.current()[0];
            Some(result)
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
            None
        } else {
            let result = self.current()[0] as i8;
            self.pos_in_data += 1;
            Some(result)
        }
    }

    pub fn peek_u16(&self) -> Option<u16> {
        if self.remain() < 2 {
            None
        } else {
            let raw_bytes = self.current()[..2].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = u16::from_le_bytes(bytes);
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
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
        if self.remain() < 2 {
            None
        } else {
            let raw_bytes = self.current()[..2].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = i16::from_le_bytes(bytes);
                    self.pos_in_data += 2;
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
        }
    }

    pub fn get_i32(&mut self) -> Option<i32> {
        if self.remain() < 4 {
            None
        } else {
            let raw_bytes = self.current()[..4].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = i32::from_le_bytes(bytes);
                    self.pos_in_data += 4;
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
        }
    }

    pub fn peek_u32(&self) -> Option<u32> {
        if self.remain() < 4 {
            None
        } else {
            let raw_bytes = self.current()[..4].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = u32::from_le_bytes(bytes);
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
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
        if self.remain() < 4 {
            None
        } else {
            let raw_bytes = self.current()[..4].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = i32::from_le_bytes(bytes);
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
        }
    }

    pub fn peek_f32(&self) -> Option<f32> {
        if self.remain() < 4 {
            None
        } else {
            let raw_bytes = self.current()[..4].try_into();
            match raw_bytes {
                Ok(bytes) => {
                    let result = f32::from_le_bytes(bytes);
                    Some(result)
                }
                Err(_) => {
                    None
                }
            }
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
            if self.current()[i as usize] != 0 {
                result = true;
                break;
            }
        }
        self.pos_in_data += bytes as usize;
        Some(result)
    }

    pub fn extract_str_l32(&mut self) -> Option<Vec<u8>> {
        let str_len = self.get_i32()? as usize;
        if str_len == 0 || str_len > self.remain() {
            return None;
        }
        let raw_str: Vec<u8>;
        if self.current()[str_len - 1] == 0 {
            raw_str = self.current()[0..str_len - 1].to_vec();
        } else {
            raw_str = self.current()[0..str_len].to_vec();
        };
        self.mov(str_len as isize);
        Some(raw_str)
    }

    pub fn extract_str_l16(&mut self) -> Option<Vec<u8>> {
        let str_len = self.get_u16()? as usize;
        if str_len == 0 || str_len > self.remain() {
            return None;
        }
        let raw_str: Vec<u8>;
        if self.current()[str_len - 1] == 0 {
            raw_str = self.current()[0..str_len - 1].to_vec();
        } else {
            raw_str = self.current()[0..str_len].to_vec();
        };
        self.mov(str_len as isize);
        Some(raw_str)
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub fn print_hex(&self, len: usize) {
        let mut i = 0;
        println!("Bytes of range [{}, {}]:", len, self.pos_in_data);
        while i < len {
            print!("{:02x} ", self.data()[self.pos_in_data + i]);
            i += 1;
        }
        println!();
    }
}
