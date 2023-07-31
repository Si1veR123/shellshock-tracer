use std::time::Duration;

use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, draw_bitmap, create_dibitmap, screen_capture, window_dimensions, object_cleanup, create_pen, bitmap_bits_to_buffer, draw_dotted_parametric_curve, draw_line, clear_bitmap};
use shellshock_tracer::{WindowsMessageLoop, Coordinate};
use shellshock_tracer::bitmap::{ARGB, Bitmap};
use shellshock_tracer::tank::Tank;
use shellshock_tracer::image_processing::find_tank;
use winapi::um::wingdi::GdiFlush;

const LOOP_DURATION: Duration = Duration::from_millis(10);

fn main() -> Result<(), &'static str> {
    let own_hwnd = create_window();
    let shellshock_hwnd = get_shellshock_window().ok_or("Shellshock application not found.")?;
    let dimensions = window_dimensions(own_hwnd).expect("Failed to get window dimensions.");

    // A large buffer that has enough size to store the pixels of the screen.
    let screen_buffer: Bitmap<'static, ARGB> = {
        let length = (dimensions.0*dimensions.1) as usize;
        let mut inner: Vec<ARGB> = Vec::with_capacity(length);
        inner.fill(0.into());
        unsafe { inner.set_len(length) };
        let slice = inner.leak();
        Bitmap { inner: slice.into(), width: dimensions.0 as usize }
    };

    let bitmap;
    let pen;
    // Bitmap and pen are deleted after use
    unsafe {
        bitmap = create_dibitmap(own_hwnd, dimensions, ARGB {r: 0, b: 0, g: 0, a: 0}).map_err(|_| "Error creating bitmap.")?;
        pen = create_pen(2, ARGB { r: 200, b: 100, g: 100, a: 255 });
    }

    // Initial data from the shellshock window
    let mut tank = Tank { screen_position: Coordinate(0, 0), angle: -77, power: 37, wind: 23 };
    
    // Main windows message pump
    WindowsMessageLoop!(own_hwnd, LOOP_DURATION, {
        //clear_bitmap(own_hwnd, bitmap, dimensions.0, dimensions.1).unwrap();
        draw_bitmap(own_hwnd, bitmap, dimensions).expect("Error drawing bitmap");
        let screen_cap = screen_capture(shellshock_hwnd).expect("Error capturing screen");
        bitmap_bits_to_buffer(shellshock_hwnd, screen_cap, dimensions, screen_buffer.inner.as_mut_ptr()).unwrap();
        let location = find_tank(&screen_buffer).unwrap();
        tank.screen_position = location;
        let closure = tank.construct_curve_function(dimensions);
        draw_dotted_parametric_curve(own_hwnd, bitmap, dimensions, pen, 4, closure).map_err(|_| "Error drawing curve.")?;
        //let loc_i32 = (location.0 as i32, location.1 as i32);
        //draw_line(own_hwnd, bitmap, dimensions, pen, loc_i32, loc_i32).unwrap();
        GdiFlush();
    });

    unsafe { object_cleanup(bitmap, pen) };

    Ok(())
}
