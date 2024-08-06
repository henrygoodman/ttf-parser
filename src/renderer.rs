use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

use crate::glyph::Glyph;

pub struct AppState {
    glyphs: Vec<Glyph>,
    canvas_dimensions: Dimensions,
    current_glyph_index: usize,
    zoom_level: f64,
}

struct Dimensions {
    width: i16,
    height: i16,
}

impl AppState {
    pub fn new(glyphs: Vec<Glyph>, width: i16, height: i16) -> Result<Self, String> {
        Ok(AppState {
            glyphs,
            canvas_dimensions: Dimensions { width, height },
            current_glyph_index: 0,
            zoom_level: 1.0,
        })
    }

    pub fn next_glyph(&mut self) {
        if self.current_glyph_index < self.glyphs.len() - 1 {
            self.current_glyph_index += 1;
        }
    }

    pub fn previous_glyph(&mut self) {
        if self.current_glyph_index > 0 {
            self.current_glyph_index -= 1;
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level += 0.1;
    }

    pub fn zoom_out(&mut self) {
        if self.zoom_level > 0.1 {
            self.zoom_level -= 0.1;
        }
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
            Color::RGB(255, 0, 0),
            Color::RGB(0, 255, 0),
            Color::RGB(0, 0, 255),
            Color::RGB(255, 255, 0),
            Color::RGB(0, 255, 255),
            Color::RGB(255, 0, 255),
        ];

        // Draw the current glyph
        let glyph = &self.glyphs[self.current_glyph_index];

        // Get bounding box for the glyph
        let (min_x, max_x, min_y, max_y) = self.get_glyph_bounding_box(glyph);

        // Calculate the glyph's width and height
        let glyph_width = max_x - min_x;
        let glyph_height = max_y - min_y;

        // Calculate the scale factor to fit the glyph within the middle box of a 3x3 grid
        let box_width = self.canvas_dimensions.width as f64 / 3.0;
        let box_height = self.canvas_dimensions.height as f64 / 3.0;
        let scale_factor_x = box_width / glyph_width as f64;
        let scale_factor_y = box_height / glyph_height as f64;
        let scale_factor = scale_factor_x.min(scale_factor_y);

        // Scale coordinates by zoom level and the calculated scale factor
        let scale = |x: i16| -> i16 { (x as f64 * self.zoom_level * scale_factor) as i16 };
        let flip_y = |y: i16| -> i16 { ((y as f64 * self.zoom_level * scale_factor) as i16) };

        // Calculate the offset to center the glyph in the middle box of the 3x3 grid
        let glyph_center_x = (min_x + max_x) / 2;
        let glyph_center_y = (min_y + max_y) / 2;

        let center_x = self.canvas_dimensions.width / 2;
        let center_y = self.canvas_dimensions.height / 2;

        // Draw contours with different colors
        let mut start = 0;
        for (contour_index, &end) in glyph.end_pts_of_contours.iter().enumerate() {
            let color = colors[contour_index % colors.len()];
            for i in start..end {
                let x1 = center_x + scale(glyph.x_coordinates[i as usize] - glyph_center_x);
                let y1 = center_y - flip_y(glyph.y_coordinates[i as usize] - glyph_center_y);
                let x2 = center_x + scale(glyph.x_coordinates[(i + 1) as usize] - glyph_center_x);
                let y2 = center_y - flip_y(glyph.y_coordinates[(i + 1) as usize] - glyph_center_y);
                canvas.line(x1, y1, x2, y2, color).unwrap();
            }
            // Close the contour
            let x1 = center_x + scale(glyph.x_coordinates[end as usize] - glyph_center_x);
            let y1 = center_y - flip_y(glyph.y_coordinates[end as usize] - glyph_center_y);
            let x2 = center_x + scale(glyph.x_coordinates[start as usize] - glyph_center_x);
            let y2 = center_y - flip_y(glyph.y_coordinates[start as usize] - glyph_center_y);
            canvas.line(x1, y1, x2, y2, color).unwrap();

            start = end + 1;
        }

        // Present the updated canvas
        canvas.present();
        Ok(())
    }
}
