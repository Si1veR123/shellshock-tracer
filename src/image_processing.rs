use std::cmp;

use crate::bitmap::{Bitmap, ARGB};

pub const TANK_HEIGHT_FRACTION: f32 = 0.019185;
pub const TANK_WIDTH_FRACTION: f32 = 0.0208333;
pub const MENU_BAR: f32 = 0.17037037;
pub const OVERLAP_PIXELS: u32 = 50;

fn tank_size_for_dimensions(dimensions: (u32, u32)) -> (u32, u32) {
    let width = dimensions.0 as f32 * TANK_WIDTH_FRACTION;
    let height = dimensions.1 as f32 * TANK_HEIGHT_FRACTION;

    (width as u32, height as u32)
}

fn pixel_score(pixel: ARGB) -> f32 {
    pixel.g.saturating_sub(pixel.r).saturating_sub(pixel.b) as f32
}

fn rolling_sum_bitmap(bitmap: &Bitmap<f32>, from: (u32, u32), to: (u32, u32), window_size: (u32, u32), overlap: u32) -> Option<(u32, u32)> {
    // from is a coordinate that must be less than to
    let game_dimensions = (to.0 - from.0, to.1 - from.1);

    let mut buffer: Vec<f32> = Vec::with_capacity((window_size.0*window_size.1) as usize);
    let mut highest_rect = (f32::MIN, None);

    for col_count in 0..((game_dimensions.0 - window_size.0) / overlap) {
        let pixel_col = col_count*overlap + from.0;

        for row_count in 0..((game_dimensions.1 - window_size.1) / overlap) {
            let pixel_row = row_count*overlap + from.1;
            
            bitmap.subrect(&mut buffer, pixel_col, pixel_row, window_size.0, window_size.1);
            let score = buffer.iter().sum();

            if score > highest_rect.0 {
                highest_rect = (score, Some((pixel_col, pixel_row)))
            }
        }
    }

    highest_rect.1
}

// relative to bottom left
pub fn find_tank(bitmap: &Bitmap<ARGB>, dimensions: (u32, u32)) -> Option<(u32, u32)> {
    let tank_size = tank_size_for_dimensions(dimensions);
    let menu_size_pixels = (dimensions.1 as f32 * MENU_BAR) as u32;
 
    let mut scores: Vec<f32> = Vec::with_capacity(bitmap.inner.len());
    for pixel in bitmap.inner.iter() {
        scores.push(pixel_score(*pixel))
    }
    let score_bitmap = Bitmap::new(scores.as_mut_slice(), bitmap.width);

    let most_likely_rect = rolling_sum_bitmap(&score_bitmap, (0, menu_size_pixels), (dimensions.0, dimensions.1), tank_size, OVERLAP_PIXELS)?;
    let expanded_from = (most_likely_rect.0.saturating_sub(tank_size.0), most_likely_rect.1.saturating_sub(tank_size.1));
    let expanded_to = (
        cmp::min(dimensions.0, expanded_from.0 + 3*tank_size.0),
        cmp::min(dimensions.1, expanded_from.1 + 3*tank_size.1)
    );
    
    let closer_rect = rolling_sum_bitmap(&score_bitmap, expanded_from, expanded_to, tank_size, 4)?;
    Some((closer_rect.0 + tank_size.0/2, closer_rect.1 + tank_size.1/2))
}
