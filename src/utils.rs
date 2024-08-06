use std::fs::File;
use std::io::Read;
use sys_info;

pub fn read_file_to_byte_array(file_path: &str) -> Vec<u8> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");
    buffer
}

pub fn get_platform_id() -> u16 {
    match sys_info::os_type().unwrap().as_str() {
        "Linux" => 3,
        "Windows" => 3,
        "MacOS" => 1,
        _ => 4,
    }
}
