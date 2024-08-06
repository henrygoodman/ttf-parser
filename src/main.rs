mod buffer;
mod glyph;
mod reader;
mod renderer;
mod utils;

use reader::{read_table_directory, read_loca_format, read_glyph_offsets, read_total_glyphs, read_glyph};
use utils::read_file_to_byte_array;
use buffer::ByteBuffer;
use renderer::AppState;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

fn main() -> Result<(), String> {
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

    let sdl_context: Sdl = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let (width, height) = (800 as i16, 800 as i16);

    let window: Window = video_subsystem
        .window("Glyph Renderer", width as u32, height as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut app_state = AppState::new(glyphs, width, height)?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    app_state.zoom_in();
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    app_state.zoom_out();
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    app_state.previous_glyph();
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    app_state.next_glyph();
                }
                _ => {}
            }
        }

        app_state.render(&mut canvas)?;
    }

    Ok(())
}
