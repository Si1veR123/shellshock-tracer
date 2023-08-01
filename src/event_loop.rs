
use std::time::Duration;

use winapi::shared::windef::{HWND, HBITMAP, HPEN};

use crate::{
    Size,
    bitmap::{
        ARGB, Bitmap
    },
    WindowsMessageLoop,
    window_winapi::{
        draw_bitmap, screen_capture, bitmap_bits_to_buffer, draw_dotted_parametric_curve, object_cleanup
    },
    image_processing::find_tank,
    tank::Tank
};

const LOOP_DURATION: Duration = Duration::from_millis(100);

pub struct BitmapBuffers<'a> {
    pub screen: Bitmap<'a, ARGB>,
    pub score: Bitmap<'a, f32>,
}

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

pub struct Config<'a> {
    pub window_handle: HWND,
    pub shellshock_handle: HWND,
    pub dimensions: Size<u32>,
    pub buffers: BitmapBuffers<'a>,
    pub windows_objects: WindowsObjects,
    pub tank: Tank,
}

pub fn event_loop<'a>(mut cfg: Config<'a>) -> Result<(), String> {
    // Main windows message pump
    WindowsMessageLoop!(own_hwnd, LOOP_DURATION, {
        draw_bitmap(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions)
            .map_err(|err| format!("Error drawing to bitmap: {err}"))?;

        let screen_cap = screen_capture(cfg.shellshock_handle)
            .map_err(|err| format!("Error capturing screen: {err}"))?;

        bitmap_bits_to_buffer(cfg.shellshock_handle, screen_cap, cfg.dimensions, cfg.buffers.screen.inner.as_mut_ptr())
            .map_err(|err| format!("Error writing bitmap bits to buffer: {err}"))?;

        let location = find_tank(&cfg.buffers.screen, &mut cfg.buffers.score)
            .ok_or_else(|| format!("Tank not found"))?;

        cfg.tank.screen_position = location;

        let parametric_path = cfg.tank.construct_curve_function(cfg.dimensions);

        draw_dotted_parametric_curve(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions, cfg.windows_objects.pen, parametric_path)
            .map_err(|err| format!("Error drawing curve: {err}"))?;
    });

    Ok(())
}