use crate::buffer::ByteBuffer;
use crate::glyph::Glyph;
use crate::table::{TableRecord, EncodingRecord, TableName, MaxpTable, HeadTable, CmapTable, CmapFormat4};
use crate::utils::get_platform_id;

pub struct FontParser {
    buffer: ByteBuffer,
    table_records: Vec<TableRecord>,
}

impl FontParser {
    pub fn new(buffer: ByteBuffer, table_records: Vec<TableRecord>) -> Self {
        Self { buffer, table_records }
    }

    pub fn read_table<T>(&mut self, table_name: TableName, read_fn: Box<dyn Fn(&mut ByteBuffer) -> T>) -> Option<T> {
        if let Some(record) = self.table_records.iter().find(|&record| &record.tag == table_name.as_tag()) {
            self.buffer.set_position(record.absolute_offset as usize);
            Some(read_fn(&mut self.buffer))
        } else {
            println!("{:?} table not found", table_name.as_tag());
            None
        }
    }

    pub fn read_maxp_table(&mut self) -> Option<MaxpTable> {
        self.read_table(TableName::Maxp, Box::new(|buffer| {
            buffer.skip_bytes(4);
            MaxpTable { num_glyphs: buffer.read_u16() }
        }))
    }

    pub fn read_head_table(&mut self) -> Option<HeadTable> {
        self.read_table(TableName::Head, Box::new(|buffer| {
            buffer.skip_bytes(2 + 2 + 4 + 4);
            let magic_number = buffer.read_u32();
            assert_eq!(magic_number, 0x5F0F3CF5);
            buffer.skip_bytes(2 + 2 + 8 + 8 + 2 + 2 + 2 + 2 + 2 + 2 + 2);
            HeadTable { index_to_loc_format: buffer.read_i16() }
        }))
    }

    pub fn read_cmap_table(&mut self) -> Option<CmapTable> {
        self.read_table(TableName::Cmap, Box::new(|buffer| {
            let table_start = buffer.current_position();
            let version = buffer.read_u16();
            assert_eq!(version, 0); // cmap header version should always be 0
            let num_tables = buffer.read_u16();
            let mut encoding_records = Vec::with_capacity(num_tables as usize);

            for _ in 0..num_tables {
                let platform_id = buffer.read_u16();
                let encoding_id = buffer.read_u16();
                let subtable_offset = buffer.read_u32();
                let subtable_absolute_offset = table_start as u32 + subtable_offset;
                encoding_records.push(EncodingRecord {
                    platform_id,
                    encoding_id,
                    subtable_absolute_offset,
                });
            }

            CmapTable {
                num_tables,
                encoding_records,
            }
        }))
    }

    fn choose_encoding_record<'a>(&self, cmap_table: &'a CmapTable) -> Option<&'a EncodingRecord> {
        let platform_id = get_platform_id();
        cmap_table.encoding_records.iter().find(|record| record.platform_id == platform_id)
            .or_else(|| {
                // Fallback to a default platform if the current one is not found
                cmap_table.encoding_records.iter().find(|record| record.platform_id == 3) // Windows platform as fallback
            })
    }

    pub fn read_cmap_subtable(&mut self, cmap_table: &CmapTable) -> Option<CmapFormat4> {
        if let Some(encoding_record) = self.choose_encoding_record(cmap_table) {
            self.buffer.set_position(encoding_record.subtable_absolute_offset as usize);
            
            let format = self.buffer.read_u16();
            if format != 4 {
                println!("Unsupported cmap format: {}", format);
                return None;
            }

            let length = self.buffer.read_u16();
            let language = self.buffer.read_u16();
            let seg_count_x2 = self.buffer.read_u16();
            let seg_count = (seg_count_x2 / 2) as usize;
            let search_range = self.buffer.read_u16();
            let entry_selector = self.buffer.read_u16();
            let range_shift = self.buffer.read_u16();

            let end_code = self.buffer.read_array::<u16>(seg_count);
            let _reserved_pad = self.buffer.read_u16();
            let start_code = self.buffer.read_array::<u16>(seg_count);
            let id_delta = self.buffer.read_array::<i16>(seg_count);
            let id_range_offset = self.buffer.read_array::<u16>(seg_count);

            let glyph_id_array_size = (length as usize - (16 + 8 * seg_count)) / 2;
            let glyph_id_array = self.buffer.read_array::<u16>(glyph_id_array_size);

            Some(CmapFormat4 {
                format,
                length,
                language,
                seg_count_x2,
                search_range,
                entry_selector,
                range_shift,
                end_code,
                start_code,
                id_delta,
                id_range_offset,
                glyph_id_array,
            })
        } else {
            None
        }
    }

    pub fn read_glyph_offsets(&mut self, num_glyphs: u16, index_to_loc_format: i16) -> Option<Vec<u32>> {
        match index_to_loc_format {
            0 => self.read_table(TableName::Loca, Box::new(move |buffer| read_loca_table_16(buffer, num_glyphs))),
            1 => self.read_table(TableName::Loca, Box::new(move |buffer| read_loca_table_32(buffer, num_glyphs))),
            _ => {
                println!("Invalid indexToLocFormat: {}", index_to_loc_format);
                None
            }
        }
    }

    pub fn read_glyph(&mut self, glyph_offsets: Option<Vec<u32>>, glyph_index: u16) -> Option<Glyph> {
        if let Some(record) = self.table_records.iter().find(|&record| &record.tag == TableName::Glyf.as_tag()) {
            if let Some(offsets) = glyph_offsets {
                let start_offset = offsets[glyph_index as usize] as usize;
                let end_offset = offsets[glyph_index as usize + 1] as usize;

                self.buffer.set_position(record.absolute_offset as usize + start_offset);
                let num_contours = self.buffer.read_i16();
                let xmin = self.buffer.read_i16();
                let ymin = self.buffer.read_i16();
                let xmax = self.buffer.read_i16();
                let ymax = self.buffer.read_i16();

                let mut end_pts_of_contours = Vec::new();
                if num_contours > 0 {
                    end_pts_of_contours = self.buffer.read_array::<u16>(num_contours as usize);
                }

                let instruction_length = self.buffer.read_u16();
                let _instructions = self.buffer.read_array::<u8>(instruction_length as usize);

                let num_points = if num_contours > 0 {
                    end_pts_of_contours[num_contours as usize - 1] + 1
                } else {
                    0
                };

                let mut flags = Vec::with_capacity(num_points as usize);
                let mut i = 0;
                while i < num_points {
                    let flag = self.buffer.read_u8();
                    flags.push(flag);

                    if (flag & 0x08) != 0 {
                        let repeat_count = self.buffer.read_u8();
                        for _ in 0..repeat_count {
                            flags.push(flag);
                        }
                        i += repeat_count as u16 + 1;
                    } else {
                        i += 1;
                    }
                }

                let mut x_coordinates = Vec::with_capacity(num_points as usize);
                let mut y_coordinates = Vec::with_capacity(num_points as usize);
                let mut previous_x = 0;
                let mut previous_y = 0;

                for &flag in &flags {
                    let x = if (flag & 0x02) != 0 {
                        let dx = self.buffer.read_u8() as i16;
                        if (flag & 0x10) != 0 {
                            previous_x + dx
                        } else {
                            previous_x - dx
                        }
                    } else {
                        if (flag & 0x10) != 0 {
                            previous_x
                        } else {
                            previous_x + self.buffer.read_i16()
                        }
                    };
                    x_coordinates.push(x);
                    previous_x = x;
                }

                for &flag in &flags {
                    let y = if (flag & 0x04) != 0 {
                        let dy = self.buffer.read_u8() as i16;
                        if (flag & 0x20) != 0 {
                            previous_y + dy
                        } else {
                            previous_y - dy
                        }
                    } else {
                        if (flag & 0x20) != 0 {
                            previous_y
                        } else {
                            previous_y + self.buffer.read_i16()
                        }
                    };
                    y_coordinates.push(y);
                    previous_y = y;
                }

                Some(Glyph {
                    num_contours,
                    xmin,
                    ymin,
                    xmax,
                    ymax,
                    end_pts_of_contours,
                    x_coordinates,
                    y_coordinates,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn read_loca_table_16(buffer: &mut ByteBuffer, num_glyphs: u16) -> Vec<u32> {
    buffer.read_array::<u16>((num_glyphs + 1) as usize).into_iter().map(|half_offset| (half_offset as u32) * 2).collect()
}

fn read_loca_table_32(buffer: &mut ByteBuffer, num_glyphs: u16) -> Vec<u32> {
    buffer.read_array::<u32>((num_glyphs + 1) as usize)
}

pub fn read_table_directory(buffer: &mut ByteBuffer) -> Vec<TableRecord> {
    let _sfnt_version = buffer.read_u32();
    let num_tables = buffer.read_u16();
    let _search_range = buffer.read_u16();
    let _entry_selector = buffer.read_u16();
    let _range_shift = buffer.read_u16();

    (0..num_tables).map(|_| {
        let tag = buffer.read_tag();
        let _checksum = buffer.read_u32();
        let absolute_offset = buffer.read_u32();
        let _length = buffer.read_u32();

        TableRecord { tag, absolute_offset }
    }).collect()
}
