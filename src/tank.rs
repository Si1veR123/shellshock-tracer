
// Constants are on a 2560x1440, 16:9 monitor
// They are scaled depending on the monitor's dimensions.
const POWER_CONSTANT: f32 = 0.995;
const WIND_CONSTANT: f32 = 0.00364;
const GRAVITY_CONSTANT: f32 = 3.04;

#[derive(Clone)]
pub struct Tank {
    pub screen_position: (u32, u32),
    pub angle: i8,
    pub power: u8,
    pub wind: i8
}

impl Tank {
    pub fn construct_curve_function(&self, width: u32, height: u32) -> Box<dyn Fn(i32) -> (i32, i32)> {
        let x_scale_ratio = width as f32 / 2560.0;
        let y_scale_ratio = height as f32 / 1440.0;

        let x_power_constant = POWER_CONSTANT * x_scale_ratio;
        let y_power_constant = POWER_CONSTANT * y_scale_ratio;

        let wind_constant = WIND_CONSTANT * x_scale_ratio;
        let gravity_constant = GRAVITY_CONSTANT * y_scale_ratio;


        let params = self.clone();
        Box::new(move |t| {
            let x_t = x_power_constant * (params.power as f32) * (params.angle as f32).to_radians().cos();
            let x_t2 = 0.5 * (params.wind as f32) * wind_constant;

            let y_t = y_power_constant * (params.power as f32) * (params.angle as f32).to_radians().sin();
            let y_t2 = -0.5 * gravity_constant;

            let t = t as f32;
            let x = x_t * t + x_t2 * t.powi(2);
            let y = y_t * t + y_t2 * t.powi(2);

            ((x as u32 + params.screen_position.0) as i32, y as i32 + params.screen_position.1 as i32)
        })
    }
}
