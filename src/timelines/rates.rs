use std::ops::Range;

pub trait WithRate<RI>
where
    Self: Sized,
    RI: Rate,
{
    type Output<RO>
    where
        RO: Rate;

    fn with_rate<F, RO>(self, f: F) -> Self::Output<RO>
    where
        RO: Rate,
        F: FnOnce(RI) -> RO;

    fn apply_rate<R>(self, applied_rate: R) -> Self::Output<Compose<R, RI>>
    where
        R: Rate,
    {
        self.with_rate(|rate| Compose(applied_rate, rate))
    }

    fn clamp(self, range: Range<f32>) -> Self::Output<Compose<Clamp, RI>> {
        self.apply_rate(Clamp(range))
    }

    fn smooth(self) -> Self::Output<Compose<Smooth, RI>> {
        self.apply_rate(Smooth)
    }
}

pub trait Rate {
    fn eval(&self, _: f32) -> f32;
}

struct Compose<R0, R1>(R0, R1);

impl<R0, R1> Rate for Compose<R0, R1>
where
    R0: Rate,
    R1: Rate,
{
    fn eval(&self, t: f32) -> f32 {
        self.1.eval(self.0.eval(t))
    }
}

struct Clamp(Range<f32>);

impl Rate for Clamp {
    fn eval(&self, t: f32) -> f32 {
        t.clamp(self.0.start, self.0.end)
    }
}

struct Smooth;

impl Rate for Smooth {
    fn eval(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

struct Smoother;

impl Rate for Smoother {
    fn eval(&self, t: f32) -> f32 {
        t * t * t * (10.0 - t * (15.0 - 6.0 * t))
    }
}

// Refer to https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html
