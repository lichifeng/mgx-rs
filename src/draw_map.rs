use crate::mapcolors::{MAP_COLORS, PLAYER_COLORS};
use crate::Parser;
use crate::Record;
use anyhow::{bail, Result};
use image::Rgb;
use imageproc::drawing::draw_filled_circle_mut;

/// Generate minimap from map data. Save it to `savename.png` 
pub fn draw_map(rec: &Record, parser: &Parser, savename: &str) -> Result<()> {
    let offset = match rec.debug.mappos {
        Some(pos) => pos,
        None => bail!("No valid map data"),
    };

    let src_width = match rec.mapx {
        Some(x) => x as u32,
        None => bail!("No mapx"),
    };

    let src_height = match rec.mapy {
        Some(y) => y as u32,
        None => bail!("No mapy"),
    };

    let rawdata = &parser.header.data()[offset as usize..];

    let mut img = image::RgbImage::new(src_width, src_height);

    let is_legacy = *rawdata.get(0).unwrap() != 0xff;
    let (terrain_offset, elevation_offset, struct_len) = if is_legacy { (0, 1, 2) } else { (1, 2, 4) };

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

    for player in &rec.players {
        if !player.isvalid() && player.initx.is_some() && player.inity.is_some() {
            continue;
        }

        if let (Some(x), Some(y), Some(color_id)) = (player.initx, player.inity, player.colorid) {
            let color = PLAYER_COLORS[color_id as usize];
            let rgb = image::Rgb([color[0] as u8, color[1] as u8, color[2] as u8]);

            // 绘制小圆 (不透明)
            let radius_small = 4;
            draw_filled_circle_mut(&mut img, (x as i32, y as i32), radius_small, rgb);
        }
    }

    img.save(savename)?;
    Ok(())
}
