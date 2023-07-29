use crate::bitmap::{Bitmap, ARGB};

const TANK_HEIGHT_FRACTION: f32 = 0.0185185;
const TANK_WIDTH_FRACTION: f32 = 0.0208333;
const MENU_BAR: f32 = 0.17037037;

fn tank_size_for_dimensions(dimensions: (u32, u32)) -> (u32, u32) {
    let width = dimensions.0 as f32 * TANK_WIDTH_FRACTION;
    let height = dimensions.1 as f32 * TANK_HEIGHT_FRACTION;

    (width as u32, height as u32)
}

fn tank_likeliness(pixels: &Vec<ARGB>) -> f32 {
    //println!("PIXELS {:?}", pixels);
    0.0
}

pub fn find_tank(bitmap: &Bitmap, dimensions: (u32, u32)) -> Option<(u32, u32)> {
    let tank_size = tank_size_for_dimensions(dimensions);
    let menu_size_pixels = (dimensions.1 as f32 * MENU_BAR) as u32;
 
    let mut buffer: Vec<ARGB> = Vec::with_capacity((tank_size.0*tank_size.1) as usize);
    let mut most_likely_rect = (0.0, None);

    for x in 0..(dimensions.0 - tank_size.0) {
        for y in 0..(dimensions.1 - tank_size.1 - menu_size_pixels) {
            bitmap.subrect(&mut buffer, x, y, tank_size.0, tank_size.1);
            let score = tank_likeliness(&buffer);

            if score > most_likely_rect.0 {
                most_likely_rect = (score, Some((x, y)))
            }
        }
    }

    most_likely_rect.1
}
