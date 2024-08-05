use std::fs::File;
use std::io::Read;

pub fn read_file_to_byte_array(file_path: &str) -> Vec<u8> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");
    buffer
}
