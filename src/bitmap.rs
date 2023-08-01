use crate::{Coordinate, Size};

// will never be used as windows is little endian
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg(target_endian = "big")]
pub struct ARGB {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg(target_endian = "little")]
pub struct ARGB {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8
}

impl ARGB {
    /// Return a u32 in the Windows API COLORREF format
    pub fn as_colorref(self) -> u32 {
        (self.b as u32) << 16 | (self.g as u32) << 8 | self.r as u32
    }

    /// Return an ARGB colour with the colour channels multiplied by the alpha channel
    pub fn as_premult_alpha(self) -> Self {
        let fraction = self.a as f32 / 255.0;
        let premult_r = (self.r as f32 * fraction) as u8;
        let premult_b = (self.b as f32 * fraction) as u8;
        let premult_g = (self.g as f32 * fraction) as u8;

        Self { a: self.a, r: premult_r, g: premult_g, b: premult_b }
    }
}

impl From<u32> for ARGB {
    fn from(value: u32) -> Self {
        // repr(C) ensures the bit layout of ARGB in big endian is
        // aaaaaaaarrrrrrrrbbbbbbbbgggggggg (32)
        // which can be transmuted to a u32
        unsafe { std::mem::transmute(value) }
    }
}

impl From<ARGB> for u32 {
    fn from(value: ARGB) -> u32 {
        // repr(C) ensures the bit layout of ARGB in big endian is
        // aaaaaaaarrrrrrrrbbbbbbbbgggggggg (32)
        // which can be transmuted to a u32
        unsafe { std::mem::transmute(value) }
    }
}

pub struct BitmapSubrectRows<'a, T> {
    inner: &'a [T],
    width: usize,
    start_coord: Coordinate<usize>,
    size: Size<usize>,              
    current_row: usize
}

impl<'a, T> BitmapSubrectRows<'a, T> {
    fn new(bitmap: &'a Bitmap<T>, start_coord: Coordinate<usize>, size: Size<usize>) -> Self {
        Self { inner: bitmap.inner, width: bitmap.width, start_coord, size, current_row: 0 }
    }
}

impl<'a, T> Iterator for BitmapSubrectRows<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.size.0 {
            return None
        }

        let row = self.current_row + self.start_coord.1;
        let index_of_row = self.width*row;
        let start_index = index_of_row+self.start_coord.0;
        self.current_row += 1;

        Some(&self.inner[start_index..start_index+self.size.1])
    }
}

pub struct Bitmap<'a, T> {
    pub inner: &'a mut [T],
    pub width: usize
}

impl<'a, T> Bitmap<'a, T> {
    pub fn new(buffer: &'a mut [T], width: usize) -> Self {
        Self { inner: buffer, width }
    }

    pub fn height(&self) -> usize {
        self.inner.len() / self.width
    }

    /// The function takes a single usize, the index of the pixel in the inner slice.
    pub fn fill_with<F: FnMut(usize) -> T>(&mut self, mut f: F) {
        for (i, pixel) in self.inner.iter_mut().enumerate() {
            *pixel = f(i);
        }
    }

    pub fn subrect(&'a self, start_coord: Coordinate<usize>, size: Size<usize>) -> BitmapSubrectRows<'a, T> {
        BitmapSubrectRows::new(self, start_coord, size)
    }
}

impl<'a, T: Clone> Bitmap<'a, T> {
    pub fn fill(&mut self, value: T) {
        self.inner.fill(value)
    }

    pub fn new_static(dimensions: Size<u32>, fill: T) -> Bitmap<'static, T> {
        let length = (dimensions.0*dimensions.1) as usize;
        let inner = vec![fill; length];
        let slice = inner.leak();
        Bitmap { inner: slice, width: dimensions.0 as usize }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn subrect_test() {
        let mut test_image = [ARGB {a: 255, r: 50, b: 50, g: 50}; 100];
        for i in 0..50 {
            test_image[i] = ARGB {a: 255, r: 100, b: 100, g: 100};
        }
        for i in [5usize, 15, 25, 35, 45, 55, 65, 75, 85, 95] {
            test_image[i] = ARGB {a: 255, r: 200, b: 200, g: 200};
        }

        let bitmap = Bitmap::new(test_image.as_mut_slice(), 10);

        let pixels: Vec<ARGB> = bitmap.subrect(Coordinate(3, 3), Size(5, 5)).flatten().cloned().collect();

        let expected = [
            ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 200, g: 200, r: 200, a: 255 },
            ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 100, g: 100, r: 100, a: 255 },
            ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 200, g: 200, r: 200, a: 255 }, ARGB { b: 100, g: 100, r: 100, a: 255 },
            ARGB { b: 100, g: 100, r: 100, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 },
            ARGB { b: 200, g: 200, r: 200, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 },
            ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 200, g: 200, r: 200, a: 255 },
            ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 },
            ARGB { b: 50, g: 50, r: 50, a: 255 }, ARGB { b: 200, g: 200, r: 200, a: 255 }, ARGB { b: 50, g: 50, r: 50, a: 255 },
            ARGB { b: 50, g: 50, r: 50, a: 255 }];

        assert_eq!(expected.as_slice(), pixels.as_slice());
    }

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
