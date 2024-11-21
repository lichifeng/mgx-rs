#[derive(Debug)]
pub struct Stream {
    pub header_decompressed: Vec<u8>,
    file_buffer: Vec<u8>,
}

impl Stream {
    pub fn new(buffer: Vec<u8>) -> Self {
        Stream {
            header_decompressed: Vec::new(),
            file_buffer: buffer,
        }
    }

    fn get_header(&self) -> &[u8] {
        &self.header_decompressed
    }
}
