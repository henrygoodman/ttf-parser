mod buffer;
mod glyph;
mod reader;
mod renderer;
mod table;
mod utils;

use reader::{FontParser, read_table_directory};
use utils::read_file_to_byte_array;
use buffer::ByteBuffer;
use renderer::AppState;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use std::env;

fn main() -> Result<(), String> {
    // Read the input string from command line arguments
    let args: Vec<String> = env::args().collect();
    let font_name = String::from("JetBrainsMono-ExtraBold.ttf");
    let input_string = if args.len() > 1 {
        args[1].clone()
    } else {
        font_name.clone()
    };

    let file_path = format!("fonts/{}", font_name);
    let bytes = read_file_to_byte_array(&file_path);
    let mut byte_buffer = ByteBuffer::new(bytes);

    let table_records = read_table_directory(&mut byte_buffer);
    let mut parser = FontParser::new(byte_buffer, table_records);

    let head_table = parser.read_head_table().expect("head table not found");
    let maxp_table = parser.read_maxp_table().expect("maxp table not found");
    let total_glyphs = maxp_table.num_glyphs;
    let glyph_offsets = parser.read_glyph_offsets(total_glyphs, head_table.index_to_loc_format);
    let cmap_table = parser.read_cmap_table().expect("cmap table not found");
    let cmap_subtable = parser.read_cmap_subtable(&cmap_table).expect("cmap subtable not found");

    // Convert input string to glyph indices
    let mut glyph_indices = Vec::new();
    for ch in input_string.chars() {
        if let Some(glyph_index) = cmap_subtable.char_to_glyph_index(ch as u16) {
            glyph_indices.push(glyph_index);
        }
    }

    // Generate glyphs for the renderer
    let mut glyphs = Vec::new();
    for glyph_index in glyph_indices {
        if let Some(glyph_data) = parser.read_glyph(glyph_offsets.clone(), glyph_index) {
            glyphs.push(glyph_data);
        }
    }

    let sdl_context: Sdl = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let (width, height) = (800 as i16, 800 as i16);

    let window: Window = video_subsystem
        .window("Glyph Renderer", width as u32, height as u32)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut app_state = AppState::new(glyphs, width, height)?;

    'running: loop {
        let mouse_state = event_pump.mouse_state();
        let (mouse_x, mouse_y) = (mouse_state.x(), mouse_state.y());

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseWheel { y, .. } => {
                    if y > 0 {
                        app_state.zoom(true, mouse_x, mouse_y);
                    } else if y < 0 {
                        app_state.zoom(false, mouse_x, mouse_y);
                    }
                },
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        app_state.start_drag(x, y);
                    }
                },
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        app_state.end_drag();
                    }
                },
                Event::MouseMotion { x, y, mousestate, .. } => {
                    if mousestate.left() {
                        app_state.update_drag(x, y);
                    }
                },
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        app_state.update_canvas_dimensions(width as i16, height as i16);
                    }
                },
                _ => {}
            }
        }

        app_state.render(&mut canvas)?;
    }

    Ok(())
}
