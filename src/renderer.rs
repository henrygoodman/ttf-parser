use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

use crate::glyph::{Glyph, GlyphCache};

pub struct AppState {
    glyphs: Vec<Vec<Glyph>>,
    canvas_dimensions: Dimensions,
    zoom_level: f64,
    debug: bool, // Enables debug visuals
    drag_start: Option<(i32, i32)>,
    offset: (f64, f64),
    line_height: f64,
    outline_thickness: i32, // Outline thickness parameter
    glyph_cache: GlyphCache, // Glyph cache
}

struct Dimensions {
    width: i16,
    height: i16,
}

impl AppState {
    pub fn new(glyphs: Vec<Vec<Glyph>>, width: i16, height: i16, debug: bool, outline_thickness: i32) -> Result<Self, String> {
        Ok(AppState {
            glyphs,
            canvas_dimensions: Dimensions { width, height },
            debug,
            zoom_level: 1.0,
            drag_start: None,
            offset: (0.0, 0.0),
            line_height: 1500.0, // Default line height
            outline_thickness, // Outline thickness parameter
            glyph_cache: GlyphCache::new(),
        })
    }

    pub fn update_canvas_dimensions(&mut self, width: i16, height: i16) {
        self.canvas_dimensions.width = width;
        self.canvas_dimensions.height = height;
    }

    pub fn zoom(&mut self, zoom_in: bool, mouse_x: i32, mouse_y: i32) {
        let zoom_factor = if zoom_in { 1.1 } else { 0.9 };
        self.zoom_level *= zoom_factor;

        let (mouse_x, mouse_y) = (mouse_x as f64, mouse_y as f64);
        let dx = (mouse_x - self.offset.0) * (1.0 - zoom_factor);
        let dy = (mouse_y - self.offset.1) * (1.0 - zoom_factor);

        self.offset.0 += dx;
        self.offset.1 += dy;

        // Update the cache with the new zoom level
        for line in &self.glyphs {
            for glyph in line {
                self.glyph_cache.update_cache(glyph, self.zoom_level);
            }
        }
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

    fn draw_bezier<T: sdl2::render::RenderTarget>(&self, canvas: &mut Canvas<T>, points: &[(i16, i16)], color: Color) -> Result<(), String> {
        if points.len() < 3 {
            return Err("Need at least 3 points to draw a quadratic Bézier curve".into());
        }

        for i in (0..points.len() - 1).step_by(2) {
            let p0 = points[i];
            let p1 = points[(i + 1) % points.len()];
            let p2 = points[(i + 2) % points.len()];

            let vx = [p0.0, p1.0, p2.0];
            let vy = [p0.1, p1.1, p2.1];

            canvas.bezier(&vx, &vy, self.outline_thickness, color).expect("Error drawing bezier");

            // Draw circles at each control point for debugging
            if self.debug {
                // println!("Points {:?}", points);
                canvas.filled_circle(vx[0] as i16, vy[0] as i16, (10.0 * self.zoom_level) as i16, Color::RGB(255, 0, 0))?;
                canvas.filled_circle(vx[1] as i16, vy[1] as i16, (5.0 * self.zoom_level) as i16, Color::RGB(0, 255, 0))?;
                canvas.filled_circle(vx[2] as i16, vy[2] as i16, (2.0 * self.zoom_level) as i16, Color::RGB(0, 0, 255))?;
            }
        }

        Ok(())
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

        let max_y_coord = self.glyphs.iter().flatten().map(|glyph| {
            let (_, _, _, max_y) = self.get_glyph_bounding_box(glyph);
            max_y as f64
        }).max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0.0);

        let mut pen_y = self.offset.1;
        for line in &self.glyphs {
            let mut pen_x = self.offset.0;

            for glyph in line {
                let (min_x, max_x, min_y, max_y) = self.get_glyph_bounding_box(glyph);

                let baseline = pen_y + (max_y_coord - max_y as f64) * self.zoom_level;

                if self.debug {
                    // println!("Glyph dimensions: width = {}, height = {}", max_x - min_x, max_y - min_y);
                    // println!("Glyph position: pen_x = {}, baseline = {}", pen_x, baseline);
                    // println!("{:?}", glyph);
                }

                if self.glyph_cache.get_cached_data(glyph.glyph_index).is_none() {
                    self.glyph_cache.update_cache(glyph, self.zoom_level);
                }

                let cached_data = self.glyph_cache.get_cached_data(glyph.glyph_index).unwrap();
                let scaled_points = &cached_data.scaled_points;
                let bounding_box = cached_data.bounding_box;

                let transformed_points: Vec<(i16, i16)> = scaled_points.iter()
                    .map(|&(x, y)| ((x as f64 + pen_x) as i16, (baseline - y as f64) as i16))
                    .collect();

                let mut start = 0;
                for (contour_index, &end) in glyph.end_pts_of_contours.iter().enumerate() {
                    assert!(
                        usize::from(end) < transformed_points.len(),
                        "Assertion failed: end={} < points.len()={}", usize::from(end), transformed_points.len()
                    );
                    let color = if self.debug { colors[contour_index % colors.len()] } else { Color::RGB(255, 255, 255) };
                    self.draw_bezier(canvas, &transformed_points[start as usize..=end as usize], color)?;
                    start = end + 1;
                }

                let dst_rect = sdl2::rect::Rect::new(
                    pen_x as i32,
                    baseline as i32,
                    (bounding_box.1 - bounding_box.0) as u32,
                    (bounding_box.3 - bounding_box.2) as u32
                );

                if self.debug {
                    canvas.set_draw_color(Color::RGB(255, 0, 0));
                    canvas.draw_rect(dst_rect).expect("Failed to draw rect outline");
                }

                pen_x += glyph.advance_width * self.zoom_level;
            }

            pen_y += self.line_height * self.zoom_level;
        }

        canvas.present();
        Ok(())
    }
}
