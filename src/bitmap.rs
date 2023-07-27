
pub struct RGBA {
    pub r: u8,
    pub b: u8,
    pub g: u8,
    pub a: u8
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

pub struct Bitmap<'a> {
    pixels: &'a mut [u32]
}

impl<'a> Bitmap<'a> {
    pub fn new(buffer: &'a mut [u32]) -> Self {
        Self { pixels: buffer }
    }

    pub fn fill(&mut self, value: RGBA) {
        self.pixels.fill(value.into())
    }

    pub fn fill_with<F: FnMut(usize) -> RGBA>(&mut self, mut f: F) {
        for (i, pixel) in self.pixels.iter_mut().enumerate() {
            *pixel = f(i).into()
        }
    }

    pub fn draw_line(&mut self, from: (u32, u32), to: (u32, u32), thickness: u32, color: RGBA) {

    }
}

impl<'a> Into<&'a mut [u32]> for &'a mut Bitmap<'a> {
    fn into(self) -> &'a mut [u32] {
        self.pixels
    }
}

impl<'a> Into<&'a [u32]> for &'a Bitmap<'a> {
    fn into(self) -> &'a [u32] {
        self.pixels
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
