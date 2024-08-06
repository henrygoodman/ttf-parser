#[derive(Debug, Clone, Copy)]
pub enum TableName {
    Maxp,
    Cmap,
    Head,
    Loca,
    Glyf,
}

impl TableName {
    pub fn as_tag(&self) -> &[u8; 4] {
        match self {
            TableName::Maxp => b"maxp",
            TableName::Cmap => b"cmap",
            TableName::Head => b"head",
            TableName::Loca => b"loca",
            TableName::Glyf => b"glyf",
        }
    }
}

#[derive(Debug)]
pub struct TableRecord {
    pub tag: [u8; 4],
    pub absolute_offset: u32,
}

#[derive(Debug)]
pub struct MaxpTable {
    pub num_glyphs: u16,
}

#[derive(Debug)]
pub struct HeadTable {
    pub index_to_loc_format: i16,
}

#[derive(Debug)]
pub struct CmapTable {
    pub num_tables: u16,
    pub encoding_records: Vec<EncodingRecord>,
}

#[derive(Debug)]
pub struct EncodingRecord {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub subtable_absolute_offset: u32,
}

#[derive(Debug)]
pub struct CmapFormat4 {
    pub format: u16,
    pub length: u16,
    pub language: u16,
    pub seg_count_x2: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
    pub end_code: Vec<u16>,
    pub start_code: Vec<u16>,
    pub id_delta: Vec<i16>,
    pub id_range_offset: Vec<u16>,
    pub glyph_id_array: Vec<u16>,
}

impl CmapFormat4 {
    pub fn char_to_glyph_index(&self, char_code: u16) -> Option<u16> {
        let seg_count = (self.seg_count_x2 / 2) as usize;

        for i in 0..seg_count {
            if char_code >= self.start_code[i] && char_code <= self.end_code[i] {
                if self.id_range_offset[i] == 0 {
                    let glyph_index = ((char_code as i32 + self.id_delta[i] as i32) % 65536) as u16;
                    return Some(glyph_index);
                } else {
                    let idx = ((self.id_range_offset[i] / 2) as usize
                        + (char_code - self.start_code[i]) as usize
                        + i) - seg_count;
                    return Some(self.glyph_id_array.get(idx).cloned().unwrap_or(0));
                }
            }
        }
        None
    }
}
