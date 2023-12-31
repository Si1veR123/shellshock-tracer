use crate::{Size, Coordinate};

// Constants are on a 2560x1440, 16:9 monitor
// They are scaled depending on the monitor's dimensions.
const WIND_CONSTANT: f32 = 0.00364;
const PATH_CONSTANT: f32 = 3.0183;

#[derive(Clone, Debug, Copy)]
pub enum Direction {
    Left,
    Right
}

impl Direction {
    pub fn as_float_multiplier(&self) -> f32 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tank {
    pub screen_position: Coordinate<u32>,
    pub angle: i8,
    pub direction: Direction,
    pub power: u8,
    pub wind: i8
}

impl Tank {
    pub fn curve_function(&self, t: i32, dimensions: Size<u32>) -> Coordinate<i32> {
        let x_scale_ratio = dimensions.0 as f32 / 2560.0;
        let y_scale_ratio = dimensions.1 as f32 / 1440.0;

        let x_power_constant = x_scale_ratio;
        let y_power_constant = y_scale_ratio;

        let wind_constant = WIND_CONSTANT * x_scale_ratio;
        let gravity_constant = PATH_CONSTANT * y_scale_ratio;

        let direction_multiplier = self.direction.as_float_multiplier();

        let x_t = x_power_constant * (self.power as f32) * (self.angle as f32).to_radians().cos();
        let x_t2 = 0.5 * (self.wind as f32) * wind_constant;

        let y_t = y_power_constant * (self.power as f32) * (self.angle as f32).to_radians().sin();
        let y_t2 = -0.5 * gravity_constant;

        let t = t as f32;
        let x = x_t * t + x_t2 * t.powi(2);
        let x_directional = x * direction_multiplier;
        let y = y_t * t + y_t2 * t.powi(2);

        Coordinate(x_directional as i32 + self.screen_position.0 as i32, y as i32 + self.screen_position.1 as i32)
    }
}
