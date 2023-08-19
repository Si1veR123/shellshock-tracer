use std::io::Write;
use std::num::ParseIntError;
use std::time::Duration;
use std::error::Error;
use std::io;
use std::sync::mpsc::channel;

use winapi::shared::windef::{HWND, HBITMAP, HPEN};

use crate::tank::Direction;
use crate::{
    Size,
    Coordinate,
    bitmap::Bitmap,
    WindowsMessageLoop,
    window_winapi::{
        draw_bitmap, screen_capture, bitmap_bits_to_buffer, draw_tank_curve, object_cleanup, clear_bitmap
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

pub fn event_loop(cfg: Config) -> Result<(), Box<dyn Error>> {
    let screen_buffer = Bitmap::new_static(cfg.dimensions, 0.into());
    let mut score_buffer = Bitmap::new_static(cfg.dimensions, 0.0);

    let mut tank = Tank { screen_position: Coordinate(0, 0), angle: -77, power: 37, wind: 23, direction: Direction::Left };

    let (tank_sender, tank_receiver) = channel();

    let _thread_handle = thread::spawn(move || {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        'input:
        loop {
            buffer.clear();
            print!("Enter (power angle wind direction:(left: 0, right: 1) ):");
            let _ = stdout.flush();

            stdin.read_line(&mut buffer).unwrap();
            let tank_data_res: Result<Vec<i8>, ParseIntError> = buffer
                .split_ascii_whitespace()
                .take(4)
                .map(|data| data.parse::<i8>())
                .collect();

            let tank = match tank_data_res {
                Ok(tank_data) => {
                    if tank_data.len() < 4 {
                        println!("All values not entered");
                        continue 'input;
                    } else {
                        let direction = if tank_data[3] == 0 { Direction::Left } else { Direction::Right };
                        Tank { screen_position: Coordinate(0, 0), power: tank_data[0] as u8, angle: tank_data[1], wind: tank_data[2], direction }
                    }
                },
                Err(_) => {
                    println!("Non-numeric values");
                    continue 'input
                }
            };
            
            let _ = tank_sender.send(tank);
        }
    });

    // Main windows message pump
    WindowsMessageLoop!(own_hwnd, LOOP_DURATION, {
        if let Ok(new_tank) = tank_receiver.try_recv() {
            tank = new_tank
        }

        let screen_cap = screen_capture(cfg.shellshock_handle)?;

        bitmap_bits_to_buffer(cfg.shellshock_handle, screen_cap, cfg.dimensions, screen_buffer.inner.as_mut_ptr())?;

        let location = find_tank(&screen_buffer, &mut score_buffer)
            .ok_or_else(|| format!("Tank not found"))?;

        tank.screen_position = location;
        draw_tank_curve(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions, cfg.windows_objects.pen, &tank)?;

        draw_bitmap(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions)?;
        clear_bitmap(cfg.window_handle, cfg.windows_objects.bitmap, cfg.dimensions)?;
    });

    Ok(())
}