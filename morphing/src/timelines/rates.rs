use std::ops::Range;

use super::alive::traits::ApplyRate;

pub trait Rate: 'static + Clone + Send + Sync {
    fn eval(&self, t: f32) -> f32;
}

#[derive(Clone)]
pub struct ClampRate(Range<f32>);

impl Rate for ClampRate {
    fn eval(&self, t: f32) -> f32 {
        t.clamp(self.0.start, self.0.end)
    }
}

pub trait Clamp: ApplyRate {
    fn clamp(self, range: Range<f32>) -> Self::Output<ClampRate> {
        self.apply_rate(ClampRate(range))
    }
}

impl<T> Clamp for T where T: ApplyRate {}

#[derive(Clone)]
pub struct SpeedRate(f32);

impl Rate for SpeedRate {
    fn eval(&self, t: f32) -> f32 {
        t * self.0
    }
}

pub trait Speed: ApplyRate {
    fn speed(self, speed: f32) -> Self::Output<SpeedRate> {
        self.apply_rate(SpeedRate(speed))
    }
}

impl<T> Speed for T where T: ApplyRate {}

#[derive(Clone)]
pub struct SmoothRate;

impl Rate for SmoothRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

pub trait Smooth: ApplyRate {
    fn smooth(self) -> Self::Output<SmoothRate> {
        self.apply_rate(SmoothRate)
    }
}

impl<T> Smooth for T where T: ApplyRate {}

#[derive(Clone)]
pub struct SmootherRate;

impl Rate for SmootherRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * t * (10.0 - t * (15.0 - 6.0 * t))
    }
}

pub trait Smoother: ApplyRate {
    fn smooth(self) -> Self::Output<SmootherRate> {
        self.apply_rate(SmootherRate)
    }
}

impl<T> Smoother for T where T: ApplyRate {}

// Refer to https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html
