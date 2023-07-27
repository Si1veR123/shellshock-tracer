use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, draw_bitmap, create_dibitmap, create_bitmap_header, create_bitmap_info, window_dimensions, object_cleanup, create_pen, draw_dotted_curve};
use shellshock_tracer::WindowsMessageLoop;
use shellshock_tracer::bitmap::{Bitmap, RGBA};

fn main() -> Result<(), &'static str> {
    let own_hwnd = create_window();
    let shellshock_hwnd = get_shellshock_window().ok_or("Shellshock application not found.")?;

    let dimensions = window_dimensions(own_hwnd).expect("Failed to get window dimensions.");
    let bitmap_header = create_bitmap_header(dimensions.0, dimensions.1);
    let bitmap_info = create_bitmap_info(bitmap_header);

    // bitmap and pen are deleted after use
    let (bitmap, buffer_ptr) = unsafe { create_dibitmap(own_hwnd, &bitmap_info).ok_or("Error creating bitmap.")? };
    let pen = unsafe { create_pen(2, RGBA { r: 100, b: 100, g: 100, a: 255 }) };

    // color buffer is valid for as long as the bitmap lives (until cleanup)
    // must ensure there are no simultaneous writes to the buffer and the bitmap, as they mutate the same data
    let mut color_buffer = Bitmap::new(buffer_ptr, (dimensions.0*dimensions.1) as usize, dimensions.0);

    unsafe {
        color_buffer.fill(RGBA {r: 0, b: 0, g: 0, a: 0});
        draw_dotted_curve(own_hwnd, bitmap, pen, 0, 2000, 10, |x| (((x as f32)/100.0).sin() * 200.0) as i32 + 200).map_err(|_| "Error drawing curve.")?;
    }

    WindowsMessageLoop!(own_hwnd, {
        draw_bitmap(own_hwnd, bitmap, dimensions.0, dimensions.1).expect("Error drawing bitmap")
    });

    unsafe { object_cleanup(bitmap, pen) };

    Ok(())
}
