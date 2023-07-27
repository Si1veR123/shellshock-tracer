use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, draw_bitmap, create_dibitmap, create_bitmap_header, create_bitmap_info, window_dimensions, bitmap_cleanup};
use shellshock_tracer::WindowsMessageLoop;
use shellshock_tracer::bitmap::{Bitmap, RGBA};

fn main() -> Result<(), &'static str> {
    let own_hwnd = create_window();
    // let shellshock_hwnd = get_shellshock_window().ok_or("Shellshock application not found.")?;

    let dimensions;
    let bitmap_info;
    let bitmap;
    let buffer_ptr;
    unsafe {
        dimensions = window_dimensions(own_hwnd).expect("Failed to get window dimensions.");

        let bitmap_header = create_bitmap_header(dimensions.0, dimensions.1);
        bitmap_info = create_bitmap_info(bitmap_header);

        (bitmap, buffer_ptr) = create_dibitmap(own_hwnd, &bitmap_info).ok_or("Error creating bitmap.")?;
    }

    // color buffer is valid for as long as the bitmap lives (until cleanup)
    let mut color_buffer = Bitmap::new(unsafe { std::slice::from_raw_parts_mut(buffer_ptr, (dimensions.0*dimensions.1) as usize) });

    color_buffer.fill_with(|i| RGBA { r: ((i as f32)/10000.0) as u8, b: 255, g: 255, a: ((i as f32)/10000.0) as u8 });

    WindowsMessageLoop!(own_hwnd, {
        draw_bitmap(own_hwnd, bitmap, (&color_buffer).into(), dimensions.1).expect("Error drawing bitmap")
    });

    unsafe { bitmap_cleanup(bitmap) };

    Ok(())
}
