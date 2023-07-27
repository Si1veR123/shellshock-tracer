
pub struct RGBA {
    pub r: u8,
    pub b: u8,
    pub g: u8,
    pub a: u8
}

impl RGBA {
    pub fn as_colorref(self) -> u32 {
        (self.b as u32) << 16 | (self.g as u32) << 8 | self.r as u32
    }
}

impl Into<u32> for RGBA {
    fn into(self) -> u32 {
        let fraction = self.a as f32 / 255.0;
        let premult_r = (self.r as f32 * fraction) as u32;
        let premult_b = (self.b as f32 * fraction) as u32;
        let premult_g = (self.g as f32 * fraction) as u32;

        (self.a as u32) << 24 | premult_r << 16 | premult_g << 8 | premult_b
    }
}

pub struct Bitmap {
    start: *mut u32,
    length: usize,
    pub width: u32
}

impl Bitmap {
    pub fn new(buffer: *mut u32, length: usize, width: u32) -> Self {
        Self { start: buffer, length, width }
    }

    pub unsafe fn fill(&mut self, value: RGBA) {
        let integer: u32 = value.into();
        let mut current_ptr = self.start;
        for _i in 0..self.length {
            *current_ptr = integer;
            current_ptr = current_ptr.add(1);
        }
    }

    /// The function takes a single usize, the index of the pixel in the inner slice.
    pub unsafe fn fill_with<F: FnMut(usize) -> RGBA>(&mut self, mut f: F) {
        let mut current_ptr = self.start;
        for i in 0..self.length {
            *current_ptr = f(i).into();
            current_ptr = current_ptr.add(1);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::RGBA;

    #[test]
    fn pre_mult_test() {
        let rgba = RGBA {r: 100, g: 100, b: 100, a: 50};
        assert_eq!(0x32131313_u32, rgba.into());
    }
}
