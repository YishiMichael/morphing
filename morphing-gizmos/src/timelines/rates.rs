use std::fmt::Debug;
use std::ops::Range;

use morphing_core::timeline::ApplyRate;
use morphing_core::traits::Rate;

macro_rules! rate {
    ($($vis:vis fn $name:ident($t:ident: $t_ty:ty$(, $rate_var:ident: $rate_var_ty:ty)*) -> $return_ty:ty $body:block)*) => {paste::paste! {$(
        $vis fn $name($t: $t_ty$(, $rate_var: $rate_var_ty)*) -> $return_ty $body

        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
        $vis struct [<$name:camel Rate>] {
            $($rate_var: $rate_var_ty,)*
        }

        impl Rate for [<$name:camel Rate>] {
            fn eval(&self, $t: $t_ty) -> $return_ty {
                $name($t$(, self.$rate_var.clone())*)
            }
        }

        $vis trait [<$name:camel>]: ApplyRate {
            fn $name(self$(, $rate_var: $rate_var_ty)*) -> Self::Output<[<$name:camel Rate>]> {
                self.apply_rate([<$name:camel Rate>] {
                    $($rate_var,)*
                })
            }
        }

        impl<T> [<$name:camel>] for T where T: ApplyRate {}
    )*}};
}

rate! {
    pub fn speed(t: f32, speed: f32) -> f32 {
        t * speed
    }

    pub fn rewind(t: f32) -> f32 {
        -t
    }

    pub fn smooth(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }

    pub fn smoother(t: f32) -> f32 {
        t * t * t * (10.0 - t * (15.0 - 6.0 * t))
    }
}

macro_rules! rate_triplets {
    ($($vis:vis $name:ident = |$t:ident| $body:expr;)*) => {paste::paste! {rate! {$(
        $vis fn [<$name _in>]($t: f32) -> f32 {
            $body
        }

        $vis fn [<$name _out>]($t: f32) -> f32 {
            1.0 - [<$name _in>](1.0 - $t)
        }

        $vis fn [<$name _in_out>]($t: f32) -> f32 {
            if $t < 0.5 {
                0.5 * [<$name _in>](2.0 * $t)
            } else {
                0.5 * ([<$name _out>](2.0 * $t - 1.0) + 1.0)
            }
        }
    )*}}};
}

// From https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html
rate_triplets! {
    pub quadratic = |t| t * t;
    pub cubic = |t| t * t * t;
    pub quartic = |t| t * t * t * t;
    pub quintic = |t| t * t * t * t * t;
    pub sine = |t| 1.0 - (std::f32::consts::FRAC_PI_2 * (1.0 - t)).sin();
    pub circular = |t| 1.0 - (1.0 - t * t).sqrt();
    pub exponential = |t| 2.0f32.powf(-10.0 * (1.0 - t));
    pub elastic = |t| (13.0 * std::f32::consts::FRAC_PI_2 * t).sin() * 2.0f32.powf(-10.0 * (1.0 - t));
    pub back = |t| t * t * t - t * (std::f32::consts::PI * t).sin();
}
