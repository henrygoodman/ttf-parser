use minifb::{Key, Window, WindowOptions};

pub struct Glyph {
    pub num_contours: i16,
    pub xmin: i16,
    pub ymin: i16,
    pub xmax: i16,
    pub ymax: i16,
    pub end_pts_of_contours: Vec<u16>,
    pub x_coordinates: Vec<i16>,
    pub y_coordinates: Vec<i16>,
}

const COLORS: [u32; 4] = [
    0xFF0000, // Red
    0x00FF00, // Green
    0x0000FF, // Blue
    0xFFFF00, // Yellow
];

pub fn draw_glyphs(glyphs: &[Glyph]) {
    let width = 600;
    let height = 800;

    let mut window = Window::new(
        "Glyph Renderer",
        width,
        height,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut buffer: Vec<u32> = vec![0; width * height];
    let mut current_glyph_index = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.iter_mut().for_each(|pixel| *pixel = 0); // Clear buffer

        // Draw the current glyph
        draw_glyph(&glyphs[current_glyph_index], &mut buffer, width, height);

        if window.is_key_down(Key::Right) {
            current_glyph_index = (current_glyph_index + 1) % glyphs.len();
            std::thread::sleep(std::time::Duration::from_millis(200)); // Debounce
        }
        if window.is_key_down(Key::Left) {
            if current_glyph_index == 0 {
                current_glyph_index = glyphs.len() - 1;
            } else {
                current_glyph_index -= 1;
            }
            std::thread::sleep(std::time::Duration::from_millis(200)); // Debounce
        }

        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

fn draw_glyph(glyph: &Glyph, buffer: &mut [u32], width: usize, height: usize) {
    let glyph_width = (glyph.xmax - glyph.xmin) as f32;
    let glyph_height = (glyph.ymax - glyph.ymin) as f32;

    let scale_x = width as f32 / glyph_width;
    let scale_y = height as f32 / glyph_height;
    let scale = scale_x.min(scale_y) * 0.9; // Apply a margin factor

    let x_offset = (width as f32 / 2.0) - (glyph_width * scale / 2.0) - (glyph.xmin as f32 * scale);
    let y_offset = (height as f32 / 2.0) - (glyph_height * scale / 2.0) - (glyph.ymin as f32 * scale);

    let mut contour_start = 0;
    for (contour_index, &end_point) in glyph.end_pts_of_contours.iter().enumerate() {
        let color = COLORS[contour_index % COLORS.len()];
        for i in contour_start..end_point as usize {
            let x1 = (glyph.x_coordinates[i] as f32 * scale + x_offset) as isize;
            let y1 = (height as f32 - (glyph.y_coordinates[i] as f32 * scale + y_offset)) as isize;
            let x2 = (glyph.x_coordinates[i + 1] as f32 * scale + x_offset) as isize;
            let y2 = (height as f32 - (glyph.y_coordinates[i + 1] as f32 * scale + y_offset)) as isize;

            draw_line(buffer, width, height, x1, y1, x2, y2, color);
        }
        let x1 = (glyph.x_coordinates[end_point as usize] as f32 * scale + x_offset) as isize;
        let y1 = (height as f32 - (glyph.y_coordinates[end_point as usize] as f32 * scale + y_offset)) as isize;
        let x2 = (glyph.x_coordinates[contour_start] as f32 * scale + x_offset) as isize;
        let y2 = (height as f32 - (glyph.y_coordinates[contour_start] as f32 * scale + y_offset)) as isize;

        draw_line(buffer, width, height, x1, y1, x2, y2, color);

        contour_start = end_point as usize + 1;
    }
}

fn draw_line(buffer: &mut [u32], width: usize, height: usize, x1: isize, y1: isize, x2: isize, y2: isize, color: u32) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut x = x1;
    let mut y = y1;

    loop {
        if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
            buffer[(y as usize) * width + (x as usize)] = color;
        }

        if x == x2 && y == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
