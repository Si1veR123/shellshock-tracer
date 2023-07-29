use std::mem::MaybeUninit;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg(target_endian = "big")]
pub struct ARGB {
    pub a: u8,
    pub r: u8,
    pub b: u8,
    pub g: u8
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg(target_endian = "little")]
pub struct ARGB {
    pub g: u8,
    pub b: u8,
    pub r: u8,
    pub a: u8
}

impl ARGB {
    pub fn as_colorref(self) -> u32 {
        (self.b as u32) << 16 | (self.g as u32) << 8 | self.r as u32
    }

    pub fn as_premult_alpha(self) -> Self {
        let fraction = self.a as f32 / 255.0;
        let premult_r = (self.r as f32 * fraction) as u8;
        let premult_b = (self.b as f32 * fraction) as u8;
        let premult_g = (self.g as f32 * fraction) as u8;

        Self { a: self.a, r: premult_r, g: premult_g, b: premult_b }
    }
}

impl Into<u32> for ARGB {
    fn into(self) -> u32 {
        // repr(C) ensures the bit layout of RGBA is
        // aaaaaaaarrrrrrrrbbbbbbbbgggggggg (32)
        // which can be transmuted to a u32
        unsafe { std::mem::transmute(self) }
    }
}

impl Into<ARGB> for u32 {
    fn into(self) -> ARGB {
        // same as Into<u32> for RGBA
        unsafe { std::mem::transmute(self) }
    }
}

pub struct Bitmap<'a> {
    pub inner: &'a mut [ARGB],
    pub width: u32
}

impl<'a> Bitmap<'a> {
    pub fn new(buffer: &'a mut [ARGB], width: u32) -> Self {
        Self { inner: buffer, width }
    }

    pub fn fill(&mut self, value: ARGB) {
        self.inner.fill(value.as_premult_alpha())
    }

    /// The function takes a single usize, the index of the pixel in the inner slice.
    pub fn fill_with<F: FnMut(usize) -> ARGB>(&mut self, mut f: F) {
        for (i, pixel) in self.inner.iter_mut().enumerate() {
            *pixel = f(i).as_premult_alpha();
        }
    }

    /// Return a subsquare with a constant number of rows
    pub fn subsrect_const<const ROWS: usize>(&self, x: u32, y: u32, columns: u32) -> [&[ARGB]; ROWS] {
        let columns = columns as usize;

        // safe for the reasons given in the MaybeUninit array initialisation example 
        let mut subsquare: [MaybeUninit<&[ARGB]>; ROWS] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..ROWS {
            let row = i as u32 + y;
            let index_of_row = self.width*row;
            let start_index = (index_of_row+x) as usize;

            let element = unsafe { subsquare.get_unchecked_mut(i) };
            element.write(&self.inner[start_index..start_index+columns]);
        }

        // copied from array_assume_init in nightly
        unsafe { (&subsquare as *const _ as *const [&[ARGB]; ROWS]).read() }
    }

    pub fn subrect(&self, buffer: &mut Vec<ARGB>, x: u32, y: u32, columns: u32, rows: u32) {
        let columns = columns as usize;

        buffer.clear();
        for i in 0..rows {
            let row = i as u32 + y;
            let index_of_row = self.width*row;
            let start_index = (index_of_row+x) as usize;

            buffer.extend_from_slice(&self.inner[start_index..start_index+columns])
        }
    }
}


#[cfg(test)]
mod tests {
    use super::ARGB;
    use std::mem;

    #[test]
    fn pre_mult_test() {
        let rgba = ARGB {r: 100, g: 100, b: 100, a: 50};
        assert_eq!(0x32131313_u32, rgba.as_premult_alpha().into());
    }

    #[test]
    fn layout_test() {
        assert_eq!(mem::size_of::<ARGB>(), 4);
        assert_eq!(mem::align_of::<ARGB>(), 1);
    }
}
