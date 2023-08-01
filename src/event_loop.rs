
use std::time::Duration;

use winapi::shared::windef::{HWND, HBITMAP, HPEN};

use crate::{
    Size,
    Coordinate,
    bitmap::Bitmap,
    WindowsMessageLoop,
    window_winapi::{
        draw_bitmap, screen_capture, bitmap_bits_to_buffer, draw_dotted_parametric_curve, object_cleanup
    },
    image_processing::find_tank,
    tank::Tank
};

const LOOP_DURATION: Duration = Duration::from_millis(100);

/// The object handles must be exclusive pointers as they are deleted after use.
pub struct WindowsObjects {
    pub bitmap: HBITMAP,
    pub pen: HPEN
}

impl Drop for WindowsObjects {
    fn drop(&mut self) {
        unsafe { object_cleanup(self.bitmap, self.pen) }
    }
}

pub struct Config {
    pub window_handle: HWND,
    pub shellshock_handle: HWND,
    pub dimensions: Size<u32>,
    pub windows_objects: WindowsObjects,
}

pub fn event_loop(cfg: Config) -> Result<(), String> {
    let screen_buffer = Bitmap::new_static(cfg.dimensions, 0.into());
    let mut score_buffer = Bitmap::new_static(cfg.dimensions, 0.0);

    let mut tank = Tank { screen_position: Coordinate(0, 0), angle: -77, power: 37, wind: 23 };

    // Main windows message pump
    WindowsMessageLoop!(own_hwnd, LOOP_DURATION, {
        draw_bitmap(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions)
            .map_err(|err| format!("Error drawing to bitmap: {err}"))?;

        let screen_cap = screen_capture(cfg.shellshock_handle)
            .map_err(|err| format!("Error capturing screen: {err}"))?;

        bitmap_bits_to_buffer(cfg.shellshock_handle, screen_cap, cfg.dimensions, screen_buffer.inner.as_mut_ptr())
            .map_err(|err| format!("Error writing bitmap bits to buffer: {err}"))?;

        let location = find_tank(&screen_buffer, &mut score_buffer)
            .ok_or_else(|| format!("Tank not found"))?;

        tank.screen_position = location;

        let parametric_path = tank.construct_curve_function(cfg.dimensions);

        draw_dotted_parametric_curve(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions, cfg.windows_objects.pen, parametric_path)
            .map_err(|err| format!("Error drawing curve: {err}"))?;
    });

    Ok(())
}