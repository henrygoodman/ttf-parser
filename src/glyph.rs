#[derive(Debug)]
pub struct Glyph {
    pub num_contours: i16,
    pub xmin: i16,
    pub ymin: i16,
    pub xmax: i16,
    pub ymax: i16,
    pub end_pts_of_contours: Vec<u16>,
    pub x_coordinates: Vec<i16>,
    pub y_coordinates: Vec<i16>,
    pub flags: Vec<u8>,
    pub processed_points: Vec<(i16, i16)>
}
