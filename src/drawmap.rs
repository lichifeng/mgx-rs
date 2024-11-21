use crate::cursor::StreamCursor;
use crate::mapcolors::{MAP_COLORS, PLAYER_COLORS};
use crate::record::Record;
use image::{Rgb, RgbImage};

/// 相比CPP的版本，这里简化了图形的旋转、放大等操作。我觉得这些操作可以在Web端用CSS实现，这里只需要生成最原始的图片即可。
pub fn draw_map(data: &StreamCursor, record: &Record, savename: &str) {
    let rawdata = &data.data()[record.debug.mappos as usize..];
    let (src_width, src_height) = (record.mapx.unwrap() as u32, record.mapy.unwrap() as u32);

    let mut img = RgbImage::new(src_width, src_height);

    let terrain_offset = 0;
    let elevation_offset = 1;
    let struct_len = 2;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cur_offset = (y * src_width + x) * struct_len;
        let rightbottom_offset = ((y + 1) * src_width + x + 1) * struct_len;
        let mut elevation = 1;

        if x < src_width - 1 && y < src_height - 1 {
            if *rawdata.get((cur_offset + elevation_offset) as usize).unwrap()
                > *rawdata.get((rightbottom_offset + elevation_offset) as usize).unwrap()
            {
                elevation = 0;
            } else if *rawdata.get((cur_offset + elevation_offset) as usize).unwrap()
                < *rawdata.get((rightbottom_offset + elevation_offset) as usize).unwrap()
            {
                elevation = 2;
            }
        }

        *pixel = Rgb(MAP_COLORS[*rawdata.get((cur_offset + terrain_offset) as usize).unwrap() as usize][elevation]);
    }

    img.save(savename).unwrap();
}
