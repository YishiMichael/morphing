use std::f32::consts::FRAC_PI_2;

use morphing_macros::rate;

#[rate(normalized, denormalized, increasing)]
fn identity(t: f32) -> f32 {
    t
}

#[rate(denormalized, increasing, assert = "speed.is_sign_positive()")]
fn speed(t: f32, speed: f32) -> f32 {
    t * speed
}

#[rate(normalized, increasing)]
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[rate(normalized, increasing)]
fn smoother(t: f32) -> f32 {
    t * t * t * (10.0 - t * (15.0 - 6.0 * t))
}

// From https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html

#[rate(normalized, increasing)]
fn quadratic(t: f32) -> f32 {
    t * t
}

#[rate(normalized, increasing)]
fn cubic(t: f32) -> f32 {
    t * t * t
}

#[rate(normalized, increasing)]
fn quartic(t: f32) -> f32 {
    t * t * t * t
}

#[rate(normalized, increasing)]
fn quintic(t: f32) -> f32 {
    t * t * t * t * t
}

#[rate(normalized, increasing)]
fn sine(t: f32) -> f32 {
    1.0 - (FRAC_PI_2 * (1.0 - t)).sin()
}

#[rate(normalized, increasing)]
fn circular(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

#[rate(normalized, increasing)]
fn exponential(t: f32) -> f32 {
    2.0f32.powf(-10.0 * (1.0 - t))
}

#[rate(normalized)]
fn elastic(t: f32) -> f32 {
    (13.0 * FRAC_PI_2 * t).sin() * exponential(t)
}

#[rate(normalized)]
fn back(t: f32) -> f32 {
    t * t * t - t * (std::f32::consts::PI * t).sin()
}

macro_rules! inherit_rate {
    ($name:ident => $name_in:ident, $name_out:ident, $name_in_out:ident) => {
        #[rate(normalized, increasing)]
        fn $name_in(t: f32) -> f32 {
            $name(t)
        }

        #[rate(normalized, increasing)]
        fn $name_out(t: f32) -> f32 {
            1.0 - $name_in(1.0 - t)
        }

        #[rate(normalized, increasing)]
        fn $name_in_out(t: f32) -> f32 {
            if t < 0.5 {
                0.5 * $name_in(2.0 * t)
            } else {
                0.5 * ($name_out(2.0 * t - 1.0) + 1.0)
            }
        }
    };
}

inherit_rate!(quadratic => quadratic_in, quadratic_out, quadratic_in_out);
inherit_rate!(cubic => cubic_in, cubic_out, cubic_in_out);
inherit_rate!(quartic => quartic_in, quartic_out, quartic_in_out);
inherit_rate!(quintic => quintic_in, quintic_out, quintic_in_out);
inherit_rate!(sine => sine_in, sine_out, sine_in_out);
inherit_rate!(circular => circular_in, circular_out, circular_in_out);
inherit_rate!(exponential => exponential_in, exponential_out, exponential_in_out);
