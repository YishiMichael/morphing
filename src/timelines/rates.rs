use std::ops::Range;

pub(crate) trait WithRate<R>
where
    R: Rate,
    Self: Sized,
{
    type Output<RO>
    where
        RO: Rate;

    fn with_rate<F, RO>(self, f: F) -> Self::Output<RO>
    where
        RO: Rate,
        F: FnOnce(R) -> RO;
}

pub trait ApplyRate<R>: WithRate<R>
where
    R: Rate,
    Self: Sized,
{
    fn apply_rate<RI>(self, applied_rate: RI) -> Self::Output<Compose<RI, R>>
    where
        RI: Rate,
    {
        self.with_rate(|rate| Compose(applied_rate, rate))
    }

    fn clamp(self, range: Range<f32>) -> Self::Output<Compose<Clamp, R>> {
        self.apply_rate(Clamp(range))
    }

    fn speed(self, speed: f32) -> Self::Output<Compose<Speed, R>> {
        self.apply_rate(Speed(speed))
    }

    fn smooth(self) -> Self::Output<Compose<Smooth, R>> {
        self.apply_rate(Smooth)
    }

    fn smoother(self) -> Self::Output<Compose<Smoother, R>> {
        self.apply_rate(Smoother)
    }
}

impl<T, RI> ApplyRate<RI> for T
where
    T: WithRate<RI>,
    RI: Rate,
{
}

// impl<T, RI> T
// where
//     T: WithRate<RI> + Sized,
//     RI: Rate,
// {
//     // add code here
// }

pub trait Rate {
    fn eval(&self, t: f32) -> f32;
}

struct Compose<R0, R1>(R0, R1);

impl<R0, R1> Rate for Compose<R0, R1>
where
    R0: Rate,
    R1: Rate,
{
    fn eval(&self, t: f32) -> f32 {
        self.0.eval(self.1.eval(t))
    }
}

struct Clamp(Range<f32>);

impl Rate for Clamp {
    fn eval(&self, t: f32) -> f32 {
        t.clamp(self.0.start, self.0.end)
    }
}

struct Speed(f32);

impl Rate for Speed {
    fn eval(&self, t: f32) -> f32 {
        t * self.0
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
