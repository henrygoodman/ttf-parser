mod buffer;
mod reader;
mod utils;
mod renderer;

use reader::{read_table_directory, read_loca_format, read_glyph_offsets, read_total_glyphs, read_glyph};
use utils::read_file_to_byte_array;
use buffer::ByteBuffer;
use renderer::{Glyph, draw_glyphs};

fn main() {
    let file_path = "fonts/JetBrainsMono-Regular.ttf";
    let bytes = read_file_to_byte_array(file_path);
    let mut byte_buffer = ByteBuffer::new(bytes);

    let table_records = read_table_directory(&mut byte_buffer);
    let loca_format = read_loca_format(&mut byte_buffer, &table_records).expect("head table not found");
    let total_glyphs = read_total_glyphs(&mut byte_buffer, &table_records).expect("maxp table not found");
    let glyph_offsets = read_glyph_offsets(&mut byte_buffer, &table_records, total_glyphs, loca_format);

    let mut glyphs = Vec::new();
    for i in 0..total_glyphs {
        if let Some(glyph_data) = read_glyph(&mut byte_buffer, &table_records, glyph_offsets.clone(), i) {
            glyphs.push(glyph_data);
        }
    }

    draw_glyphs(&glyphs);

    println!("Remaining bytes in buffer: {}", byte_buffer.remaining());
}
