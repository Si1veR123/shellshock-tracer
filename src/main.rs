use std::error::Error;

use shellshock_tracer::window_winapi::{create_window, get_shellshock_window, create_dibitmap, window_dimensions, create_pen};
use shellshock_tracer::bitmap::ARGB;
use shellshock_tracer::event_loop::{WindowsObjects, Config, event_loop};

fn main() -> Result<(), Box<dyn Error>> {
    let own_hwnd = create_window()?;

    let shellshock_hwnd = get_shellshock_window()
        .ok_or("Shellshock application not found.")?;

    let dimensions = unsafe { window_dimensions(own_hwnd)? };

    let windows_objects = WindowsObjects {
        bitmap: unsafe { create_dibitmap(own_hwnd, dimensions, 0.into())? },
        pen: create_pen(2, ARGB { r: 200, b: 100, g: 100, a: 255 })?,
    };
    
    let config = Config {
        window_handle: own_hwnd,
        shellshock_handle: shellshock_hwnd,
        dimensions,
        windows_objects
    };

    event_loop(config)?;

    Ok(())
}
