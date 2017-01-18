use std::io::{self, BufReader, Bytes};
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::str;

const DEFAULT_BUF_SIZE: usize = 64 * 1024;

pub struct Scanner<R: Read> {
        reader: R,
}

impl<R: Read> Scanner<R> {
        pub fn new(reader: R) -> Self {
                Scanner {
                        reader: reader,
                }
        }

        pub fn scan(&mut self) -> io::Result<Vec<u8>> {
                let mut contents: Vec<u8> = Vec::new();

                let mut buffer = [0; DEFAULT_BUF_SIZE];
                let mut total_count = 0;
                let mut data_size_within_buffer = 0;

                loop {
                        data_size_within_buffer = self.reader.read(&mut buffer).unwrap();
                        for b in 0..data_size_within_buffer {
                                contents.push(buffer[b]);
                        }
                        if data_size_within_buffer < DEFAULT_BUF_SIZE {
                                break;
                        }
                } 

                Ok(contents)
        }
}

#[cfg(test)]
mod tests {

        use std::fs;
        use super::*;

        #[test]
        fn read_test_string() {
                let mut scanner = Scanner::new(&b"Hello world!"[..]);

                let contents = scanner.scan().unwrap();

                assert_eq!(contents.len(), 12);
        }

        #[test]
        fn read_test_file_under_buffer_size() {
                let mut scanner = Scanner::new(fs::File::open("./test_assets/foo.txt").unwrap());

                let contents = scanner.scan().unwrap();

                assert_eq!(contents.len(), 3);
        }
}
