use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, create_dibitmap, window_dimensions, create_pen};
use shellshock_tracer::Coordinate;
use shellshock_tracer::bitmap::{ARGB, Bitmap};
use shellshock_tracer::tank::Tank;
use shellshock_tracer::event_loop::{BitmapBuffers, WindowsObjects, Config, event_loop};

fn main() -> Result<(), String> {
    let own_hwnd = create_window().map_err(|err| format!("Error creating window: {err}"))?;
    let shellshock_hwnd = get_shellshock_window().ok_or("Shellshock application not found.")?;
    let dimensions = unsafe { window_dimensions(own_hwnd).expect("Failed to get window dimensions.") };

    // Large buffers that has enough size to store the pixels of the screen.
    let buffers = BitmapBuffers {
        screen: Bitmap::new_static(dimensions, 0.into()),
        score: Bitmap::new_static(dimensions, 0.0),
    };

    let windows_objects = WindowsObjects {
        bitmap: unsafe { create_dibitmap(own_hwnd, dimensions, 0.into()).map_err(|_| "Error creating bitmap.")? },
        pen: unsafe { create_pen(2, ARGB { r: 200, b: 100, g: 100, a: 255 }) },
    };

    let tank = Tank { screen_position: Coordinate(0, 0), angle: -77, power: 37, wind: 23 };

    let config = Config {
        window_handle: own_hwnd,
        shellshock_handle: shellshock_hwnd,
        dimensions,
        buffers,
        windows_objects,
        tank,
    };

    event_loop(config)?;

    Ok(())
}
