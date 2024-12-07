use crate::mapcolors::{MAP_COLORS, PLAYER_COLORS};
use crate::parser::Parser;
use anyhow::{bail, Result};
use image::{Rgb, RgbImage};

impl Parser {
    /// Use this after `.parse_to(some_rec)`   
    /// 相比CPP的版本，这里简化了图形的旋转、放大等操作。我觉得这些操作可以在Web端用CSS实现，这里只需要生成最原始的图片即可。
    pub fn draw_map(self: &Self, savename: &str) -> Result<()> {
        if self.mapattr.0.is_none() || self.mapattr.1.is_none() || self.mapattr.2.is_none() {
            bail!("No valid map data found");
        }

        let rawdata = &self.header.data()[self.mapattr.0.unwrap() as usize..];
        let (src_width, src_height) = (self.mapattr.1.unwrap() as u32, self.mapattr.2.unwrap() as u32);

        let mut img = RgbImage::new(src_width, src_height);

        let not_legacy_tile = *rawdata.get(0).unwrap() == 0xff;
        let (terrain_offset, elevation_offset, struct_len) = if not_legacy_tile {
            (1, 2, 4)
        } else {
            (0, 1, 2)
        };

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

        img.save(savename)?;
        Ok(())
    }
}
