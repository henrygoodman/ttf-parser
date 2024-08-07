use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

use crate::glyph::Glyph;

pub struct AppState {
    glyphs: Vec<Vec<Glyph>>,
    canvas_dimensions: Dimensions,
    zoom_level: f64,
    drag_start: Option<(i32, i32)>,
    offset: (f64, f64),
    padding: f64,
    line_height: f64,
}

struct Dimensions {
    width: i16,
    height: i16,
}

impl AppState {
    pub fn new(glyphs: Vec<Vec<Glyph>>, width: i16, height: i16) -> Result<Self, String> {
        Ok(AppState {
            glyphs,
            canvas_dimensions: Dimensions { width, height },
            zoom_level: 1.0,
            drag_start: None,
            offset: (0.0, 0.0),
            padding: 10.0,          // FIXME: Get proper horizontal spacing
            line_height: 1000.0,    // FIXME: Get proper vertical spacing
        })
    }

    pub fn update_canvas_dimensions(&mut self, width: i16, height: i16) {
        self.canvas_dimensions.width = width;
        self.canvas_dimensions.height = height;
    }

    pub fn zoom(&mut self, zoom_in: bool, mouse_x: i32, mouse_y: i32) {
        let zoom_factor = if zoom_in { 1.1 } else { 0.9 };
        self.zoom_level *= zoom_factor;

        // Calculate the offset change to keep the zoom centered around the mouse position
        let (mouse_x, mouse_y) = (mouse_x as f64, mouse_y as f64);

        // Adjust offset relative to the mouse position and zoom level
        let dx = (mouse_x - self.offset.0) * (1.0 - zoom_factor);
        let dy = (mouse_y - self.offset.1) * (1.0 - zoom_factor);

        self.offset.0 += dx;
        self.offset.1 += dy;
    }

    pub fn start_drag(&mut self, x: i32, y: i32) {
        self.drag_start = Some((x, y));
    }

    pub fn update_drag(&mut self, x: i32, y: i32) {
        if let Some((start_x, start_y)) = self.drag_start {
            let dx = (x - start_x) as f64;
            let dy = (y - start_y) as f64;
            self.offset.0 += dx;
            self.offset.1 += dy;
            self.drag_start = Some((x, y));
        }
    }

    pub fn end_drag(&mut self) {
        self.drag_start = None;
    }

    fn get_glyph_bounding_box(&self, glyph: &Glyph) -> (i16, i16, i16, i16) {
        let min_x = *glyph.x_coordinates.iter().min().unwrap_or(&0);
        let max_x = *glyph.x_coordinates.iter().max().unwrap_or(&0);
        let min_y = *glyph.y_coordinates.iter().min().unwrap_or(&0);
        let max_y = *glyph.y_coordinates.iter().max().unwrap_or(&0);
        (min_x, max_x, min_y, max_y)
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Clear the canvas with a black background
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Colors for different contours
        let colors = [
            Color::RGB(255, 255, 255),  // White
            Color::RGB(0, 255, 127),    // Spring Green
            Color::RGB(0, 191, 255),    // Deep Sky Blue
            Color::RGB(255, 165, 0),    // Orange
            Color::RGB(138, 43, 226),   // Blue Violet
            Color::RGB(255, 20, 147),   // Deep Pink
        ];

        // Determine the overall glyph height to maintain baseline alignment
        let max_y_coord = self.glyphs.iter().flatten().map(|glyph| {
            let (_, _, _, max_y) = self.get_glyph_bounding_box(glyph);
            max_y as f64
        }).max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0.0);

        // Determine the fixed width for all glyphs based on the widest glyph
        let fixed_width = self.glyphs.iter().flatten().map(|glyph| {
            let (min_x, max_x, _, _) = self.get_glyph_bounding_box(glyph);
            (max_x - min_x) as f64
        }).max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0.0);

        // Draw glyphs
        let mut pen_y = self.offset.1;
        for line in &self.glyphs {
            let mut pen_x = self.offset.0;

            for glyph in line {
                // Get bounding box for the glyph
                let (min_x, max_x, min_y, max_y) = self.get_glyph_bounding_box(glyph);

                // Calculate the glyph's width and height
                let glyph_width = (max_x - min_x) as f64;
                let glyph_height = (max_y - min_y) as f64;

                // Calculate baseline for the current glyph
                let baseline = pen_y + (max_y_coord - max_y as f64) * self.zoom_level;

                // Debug: Output the dimensions and position of the glyph
                // println!("Glyph dimensions: width = {}, height = {}", glyph_width, glyph_height);
                // println!("Glyph position: pen_x = {}, baseline = {}", pen_x, baseline);
                // println!("{:?}", glyph);

                // Scale coordinates by zoom level
                let scale = |x: i16| -> i16 { (x as f64 * self.zoom_level) as i16 };
                let flip_y = |y: i16| -> i16 { (y as f64 * self.zoom_level) as i16 };

                // Draw contours with different colors
                let mut start = 0;
                for (contour_index, &end) in glyph.end_pts_of_contours.iter().enumerate() {
                    let color = colors[contour_index % colors.len()];
                    for i in start..end {
                        let x1 = (pen_x + scale(glyph.x_coordinates[i as usize] - min_x) as f64) as i16;
                        let y1 = (baseline - flip_y(glyph.y_coordinates[i as usize] - max_y) as f64) as i16;
                        let x2 = (pen_x + scale(glyph.x_coordinates[(i + 1) as usize] - min_x) as f64) as i16;
                        let y2 = (baseline - flip_y(glyph.y_coordinates[(i + 1) as usize] - max_y) as f64) as i16;
                        canvas.line(x1, y1, x2, y2, color).unwrap();
                    }
                    // Close the contour
                    let x1 = (pen_x + scale(glyph.x_coordinates[end as usize] - min_x) as f64) as i16;
                    let y1 = (baseline - flip_y(glyph.y_coordinates[end as usize] - max_y) as f64) as i16;
                    let x2 = (pen_x + scale(glyph.x_coordinates[start as usize] - min_x) as f64) as i16;
                    let y2 = (baseline - flip_y(glyph.y_coordinates[start as usize] - max_y) as f64) as i16;
                    canvas.line(x1, y1, x2, y2, color).unwrap();

                    start = end + 1;
                }

                // Move pen to the next glyph position with padding
                pen_x += (fixed_width + self.padding) * self.zoom_level;
            }

            // Move pen to the next line
            pen_y += (self.line_height + self.padding) * self.zoom_level;
        }

        // Present the updated canvas
        canvas.present();
        Ok(())
    }
}