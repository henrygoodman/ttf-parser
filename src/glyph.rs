use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Glyph {
    pub glyph_index: u16,
    pub num_contours: i16,
    pub xmin: i16,
    pub ymin: i16,
    pub xmax: i16,
    pub ymax: i16,
    pub end_pts_of_contours: Vec<u16>,
    pub x_coordinates: Vec<i16>,
    pub y_coordinates: Vec<i16>,
    pub flags: Vec<u8>,
    pub processed_points: Vec<(i16, i16)>, // Combines actual points and 'implied' bezier control points
    pub advance_width: f64,
}

pub struct GlyphCache {
    pub cache: HashMap<u16, CachedGlyphData>,
}

pub struct CachedGlyphData {
    pub scaled_points: Vec<(i16, i16)>,
    pub bounding_box: (i16, i16, i16, i16),
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn update_cache(&mut self, glyph: &Glyph, zoom_level: f64) {
        let scale = |x: i16| -> i16 { (x as f64 * zoom_level) as i16 };
        let flip_y = |y: i16| -> i16 { (y as f64 * zoom_level) as i16 };

        let (min_x, max_x, min_y, max_y) = (
            *glyph.x_coordinates.iter().min().unwrap_or(&0),
            *glyph.x_coordinates.iter().max().unwrap_or(&0),
            *glyph.y_coordinates.iter().min().unwrap_or(&0),
            *glyph.y_coordinates.iter().max().unwrap_or(&0),
        );

        let scaled_points = glyph.processed_points.iter()
            .map(|&(x, y)| (scale(x - min_x), flip_y(y - max_y)))
            .collect();

        let bounding_box = (
            scale(min_x),
            scale(max_x),
            flip_y(min_y),
            flip_y(max_y),
        );

        self.cache.insert(glyph.glyph_index, CachedGlyphData {
            scaled_points,
            bounding_box,
        });
    }

    pub fn get_cached_data(&self, glyph_index: u16) -> Option<&CachedGlyphData> {
        self.cache.get(&glyph_index)
    }
}

