use std::cmp;

use crate::bitmap::{Bitmap, ARGB};
use crate::{Coordinate, Size};

pub const TANK_HEIGHT_FRACTION: f32 = 0.019535;
pub const TANK_WIDTH_FRACTION: f32 = 0.01736;
pub const MENU_BAR: f32 = 0.17037037;
pub const OVERLAP_PIXELS: usize = 50;

fn tank_size_for_dimensions(dimensions: Size<usize>) -> Size<usize> {
    let width = dimensions.0 as f32 * TANK_WIDTH_FRACTION;
    let height = dimensions.1 as f32 * TANK_HEIGHT_FRACTION;

    Size(width as usize, height as usize)
}

fn pixel_score(pixel: ARGB) -> f32 {
    pixel.g.saturating_sub(pixel.r).saturating_sub(pixel.b) as f32
}

fn rolling_sum_bitmap(bitmap: &Bitmap<f32>, from: Coordinate<usize>, to: Coordinate<usize>, window_size: Size<usize>, overlap: usize) -> Option<Coordinate<usize>> {
    // from is a coordinate that must be less than to
    let game_dimensions = (to.0 - from.0, to.1 - from.1);
    let mut highest_rect = (f32::MIN, None);

    for col_count in 0..((game_dimensions.0 - window_size.0) / overlap) {
        let pixel_col = col_count*overlap + from.0;

        for row_count in 0..((game_dimensions.1 - window_size.1) / overlap) {
            let pixel_row = row_count*overlap + from.1;
            
            let rows = bitmap.subrect(Coordinate(pixel_col, pixel_row), Size(window_size.0, window_size.1));
            let score = rows.fold(0.0, |acc, row| acc + row.iter().sum::<f32>());

            if score > highest_rect.0 {
                highest_rect = (score, Some(Coordinate(pixel_col, pixel_row)))
            }
        }
    }

    highest_rect.1
}

// relative to bottom left
pub fn find_tank(bitmap: &Bitmap<ARGB>, score_bitmap: &mut Bitmap<f32>) -> Option<Coordinate<u32>> {
    let dimensions = Size(bitmap.width, bitmap.height());

    let tank_size = tank_size_for_dimensions(dimensions);
    let menu_size_pixels = (dimensions.1 as f32 * MENU_BAR) as usize;
 
    for (i, pixel) in score_bitmap.inner.iter_mut().enumerate() {
        *pixel = pixel_score(bitmap.inner[i])
    }

    let most_likely_rect = rolling_sum_bitmap(&score_bitmap, Coordinate(0, menu_size_pixels), Coordinate(dimensions.0, dimensions.1), tank_size, OVERLAP_PIXELS)?;
    let expanded_from = Coordinate(most_likely_rect.0.saturating_sub(tank_size.0), most_likely_rect.1.saturating_sub(tank_size.1));
    let expanded_to = Coordinate(
        cmp::min(dimensions.0, expanded_from.0 + 3*tank_size.0),
        cmp::min(dimensions.1, expanded_from.1 + 3*tank_size.1)
    );
    
    let closer_rect = rolling_sum_bitmap(&score_bitmap, expanded_from, expanded_to, tank_size, 1)?;
    Some(Coordinate((closer_rect.0 + tank_size.0/2) as u32, (closer_rect.1 + tank_size.1/2) as u32))
}
