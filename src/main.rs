use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, draw_bitmap, create_dibitmap, screen_capture, window_dimensions, object_cleanup, create_pen, bitmap_bits_to_buffer, draw_dotted_parametric_curve, bitmap_to_clipboard};
use shellshock_tracer::WindowsMessageLoop;
use shellshock_tracer::bitmap::{ARGB, Bitmap};
use shellshock_tracer::tank::Tank;
use shellshock_tracer::image_processing::find_tank;
use winapi::ctypes::c_void;
use winapi::um::wingdi::DeleteObject;

fn main() -> Result<(), &'static str> {
    let own_hwnd = create_window();
    let shellshock_hwnd = get_shellshock_window().ok_or("Shellshock application not found.")?;
    let dimensions = window_dimensions(own_hwnd).expect("Failed to get window dimensions.");

    // A large buffer that has enough size to store the pixels of the screen.
    let screen_buffer: Bitmap<'static> = {
        let length = (dimensions.0*dimensions.1) as usize;
        let mut inner: Vec<ARGB> = Vec::with_capacity(length);
        inner.fill(0.into());
        unsafe { inner.set_len(length) };
        let slice = inner.leak();
        Bitmap { inner: slice.into(), width: dimensions.0 }
    };

    let bitmap;
    let pen;
    // Bitmap and pen are deleted after use
    unsafe {
        bitmap = create_dibitmap(own_hwnd, dimensions, ARGB {r: 0, b: 0, g: 0, a: 0}).map_err(|_| "Error creating bitmap.")?;
        pen = create_pen(2, ARGB { r: 200, b: 100, g: 100, a: 255 });
    }

    // Initial data from the shellshock window
    let tank = Tank { screen_position: (100, 245), angle: 10, power: 100, wind: 0 };
    let closure = tank.construct_curve_function(dimensions.0, dimensions.1);

    unsafe { draw_dotted_parametric_curve(own_hwnd, bitmap, pen, dimensions.0 as i32, dimensions.1 as i32, 4, closure).map_err(|_| "Error drawing curve.")? };

    // Main windows message pump
    WindowsMessageLoop!(own_hwnd, {
        draw_bitmap(own_hwnd, bitmap, dimensions.0, dimensions.1).expect("Error drawing bitmap");

        let screen_cap = screen_capture(shellshock_hwnd).expect("Error capturing screen");
        let _ = bitmap_to_clipboard(screen_cap);
        let _ = bitmap_bits_to_buffer(shellshock_hwnd, screen_cap, dimensions.0, dimensions.1, screen_buffer.inner.as_mut_ptr());
        find_tank(&screen_buffer, dimensions);
        DeleteObject(screen_cap as *mut c_void);
    });

    unsafe { object_cleanup(bitmap, pen) };

    Ok(())
}
