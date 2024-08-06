use crate::buffer::ByteBuffer;
use crate::glyph::Glyph;

#[derive(Debug)]
pub struct TableRecord {
    pub tag: [u8; 4],
    pub absolute_offset: u32,
}

pub fn read_table_directory(buffer: &mut ByteBuffer) -> Vec<TableRecord> {
    // -- Table Directory -- https://learn.microsoft.com/en-us/typography/opentype/spec/otff#table-directory
    // sfntVersion      (u32) - Font version
    // numTables        (u16) - Number of tables
    // searchRange      (u16) - Maximum power of 2 <= numTables * 16
    // entrySelector    (u16) - Log2(maximum power of 2 <= numTables)
    // rangeShift       (u16) - NumTables * 16 - searchRange
    let sfnt_version = buffer.read_u32();
    assert!(sfnt_version == 0x00010000 || sfnt_version == 0x4f54544F);

    let num_tables = buffer.read_u16();

    println!("Num tables: {:?}", num_tables);

    buffer.skip_bytes(6);
    read_table_records(buffer, num_tables)
}

fn read_table_records(buffer: &mut ByteBuffer, num_tables: u16) -> Vec<TableRecord> {
    let mut table_records = Vec::with_capacity(num_tables as usize);
    let base_offset = buffer.current_position();
    for _ in 0..num_tables {
        let tag = buffer.read_tag();
        let _checksum = buffer.read_u32();
        let absolute_offset = buffer.read_u32();
        let _length = buffer.read_u32(); 

        // println!("Tag: {} Position: {:?}", std::str::from_utf8(&tag).expect("Invalid UTF-8"), absolute_offset);

        table_records.push(TableRecord { tag, absolute_offset });
    }
    table_records
}

// maxp table parsing
pub fn read_total_glyphs(buffer: &mut ByteBuffer, table_records: &[TableRecord]) -> Option<u16> {
    if let Some(record) = table_records.iter().find(|&record| &record.tag == b"maxp") {
        buffer.set_position(record.absolute_offset as usize);
        buffer.skip_bytes(4); // Skip version (4 bytes)
        let num_glyphs = buffer.read_u16();
        
        println!("Total Glyphs: {:?}", num_glyphs);
        
        Some(num_glyphs)
    } else {
        
        println!("maxp table not found");
        
        None
    }
}

// head table parsing
pub fn read_loca_format(buffer: &mut ByteBuffer, table_records: &[TableRecord]) -> Option<i16> {
    if let Some(record) = table_records.iter().find(|&record| &record.tag == b"head") {
        buffer.set_position(record.absolute_offset as usize);
        buffer.skip_bytes(2 + 2 + 4 + 4);
        let magic_number = buffer.read_u32();
        assert_eq!(magic_number, 0x5F0F3CF5);
        buffer.skip_bytes(2 + 2 + 8 + 8 + 2 + 2 + 2 + 2 + 2 + 2 + 2);
        let index_to_loc_format = buffer.read_i16();
        
        // println!("Ioca format: {:?}", index_to_loc_format); // Should be 0 or 1
        
        Some(index_to_loc_format)
    } else {
        
        println!("head table not found");
        
        None
    }
}

// loca table parsing
pub fn read_glyph_offsets(buffer: &mut ByteBuffer, table_records: &[TableRecord], num_glyphs: u16, index_to_loc_format: i16) -> Option<Vec<u32>> {
    if let Some(record) = table_records.iter().find(|&record| &record.tag == b"loca") {
        buffer.set_position(record.absolute_offset as usize);

        let glyph_offsets = match index_to_loc_format {
            0 => {
                // Short format (16-bit)
                buffer.read_array::<u16>((num_glyphs + 1) as usize)
                      .into_iter()
                      .map(|half_offset| (half_offset as u32) * 2)
                      .collect()
            }
            1 => {
                // Long format (32-bit)
                buffer.read_array::<u32>((num_glyphs + 1) as usize)
            }
            _ => {
                println!("Invalid indexToLocFormat: {}", index_to_loc_format);
                return None;
            }
        };

        Some(glyph_offsets)
    } else {
        println!("loca table not found");
        None
    }
}

// glyph table parsing
pub fn read_glyph_table(buffer: &mut ByteBuffer, table_records: &[TableRecord], total_glyphs: u16) {
    if let Some(record) = table_records.iter().find(|&record| &record.tag == b"glyf") {
        buffer.set_position(record.absolute_offset as usize);
        for i in 0..1 {
            
            print!("Glyph {:?}: ", i);
            
            let num_contours = read_glyph_header(buffer)[0];
            read_glyph_data(buffer, num_contours);
        }
    } else {
        
        println!("glyf table not found");
    }
}

pub fn read_glyph_header(buffer: &mut ByteBuffer) -> [i16; 5] {
    let num_contours = buffer.read_i16();
    let xmin = buffer.read_i16();
    let ymin = buffer.read_i16();
    let xmax = buffer.read_i16();
    let ymax = buffer.read_i16();
    
    println!("{:?} contours, x_min: {:?}, y_min: {:?}, x_max: {:?}, y_max: {:?}", num_contours, xmin, ymin, xmax, ymax);
    
    [num_contours, xmin, ymin, xmax, ymax]
}

pub fn read_glyph(buffer: &mut ByteBuffer, table_records: &[TableRecord], glyph_offsets: Option<Vec<u32>>, glyph_index: u16) -> Option<Glyph> {
    if let Some(record) = table_records.iter().find(|&record| &record.tag == b"glyf") {
        if let Some(offsets) = glyph_offsets {
            let start_offset = offsets[glyph_index as usize] as usize;
            let end_offset = offsets[glyph_index as usize + 1] as usize;

            buffer.set_position(record.absolute_offset as usize + start_offset);
            let num_contours = buffer.read_i16();
            let xmin = buffer.read_i16();
            let ymin = buffer.read_i16();
            let xmax = buffer.read_i16();
            let ymax = buffer.read_i16();

            let mut end_pts_of_contours = Vec::new();
            if num_contours > 0 {
                end_pts_of_contours = buffer.read_array::<u16>(num_contours as usize);
            }

            let instruction_length = buffer.read_u16();
            let _instructions = buffer.read_array::<u8>(instruction_length as usize);

            let num_points = if num_contours > 0 {
                end_pts_of_contours[num_contours as usize - 1] + 1
            } else {
                0
            };

            let mut flags = Vec::with_capacity(num_points as usize);
            let mut i = 0;
            while i < num_points {
                let flag = buffer.read_u8();
                flags.push(flag);

                if (flag & 0x08) != 0 {
                    let repeat_count = buffer.read_u8();
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
                    let dx = buffer.read_u8() as i16;
                    if (flag & 0x10) != 0 {
                        previous_x + dx
                    } else {
                        previous_x - dx
                    }
                } else {
                    if (flag & 0x10) != 0 {
                        previous_x
                    } else {
                        previous_x + buffer.read_i16()
                    }
                };
                x_coordinates.push(x);
                previous_x = x;
            }

            for &flag in &flags {
                let y = if (flag & 0x04) != 0 {
                    let dy = buffer.read_u8() as i16;
                    if (flag & 0x20) != 0 {
                        previous_y + dy
                    } else {
                        previous_y - dy
                    }
                } else {
                    if (flag & 0x20) != 0 {
                        previous_y
                    } else {
                        previous_y + buffer.read_i16()
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

pub fn read_glyph_data(buffer: &mut ByteBuffer, num_contours: i16) {
    if num_contours <= 0 {
        println!("No contours");
        return;
    }

    let end_points = buffer.read_array::<u16>(num_contours as usize);
    println!("End Points: {:?}", &end_points);
    let instruction_length = buffer.read_u16();
    let _instructions = buffer.read_array::<u8>(instruction_length as usize);

    let num_points = end_points[num_contours as usize - 1] + 1;
    let mut flags = Vec::with_capacity(num_points as usize);

    let mut i = 0;
    while i < num_points {
        let flag = buffer.read_u8();
        flags.push(flag);

        if (flag & 0x08) != 0 {
            let repeat_count = buffer.read_u8();
            println!("Repeat count found {:?}", repeat_count);
            for _ in 0..repeat_count {
                flags.push(flag);
            }
            i += repeat_count as u16 + 1;
        } else {
            i += 1;
        }
    }

    println!("Flags: {:?}", &flags);

    let mut x_coordinates = Vec::with_capacity(num_points as usize);
    let mut y_coordinates = Vec::with_capacity(num_points as usize);
    let mut previous_x = 0;
    let mut previous_y = 0;

    for &flag in &flags {
        let x = if (flag & 0x02) != 0 {
            let dx = buffer.read_u8() as i16;
            if (flag & 0x10) != 0 {
                previous_x + dx
            } else {
                previous_x - dx
            }
        } else {
            if (flag & 0x10) != 0 {
                previous_x
            } else {
                previous_x + buffer.read_i16()
            }
        };
        x_coordinates.push(x);
        previous_x = x;
    }

    for &flag in &flags {
        let y = if (flag & 0x04) != 0 {
            let dy = buffer.read_u8() as i16;
            if (flag & 0x20) != 0 {
                previous_y + dy
            } else {
                previous_y - dy
            }
        } else {
            if (flag & 0x20) != 0 {
                previous_y
            } else {
                previous_y + buffer.read_i16()
            }
        };
        y_coordinates.push(y);
        previous_y = y;
    }

    println!("X Coordinates: {:?}", x_coordinates);
    println!("Y Coordinates: {:?}", y_coordinates);

    println!("Finished parsing glyph data!");
}
