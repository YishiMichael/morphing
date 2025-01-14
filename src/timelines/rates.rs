use std::ops::Range;

pub trait Rate: 'static {
    fn eval(&self, t: f32) -> f32;
}

pub trait ApplyRate {
    type Output<R>
    where
        R: Rate;

    fn apply_rate<R>(self, rate: R) -> Self::Output<R>
    where
        R: Rate;
}

pub trait ApplyRateChain {
    type InRate: Rate;
    type Partial;
    type Output<RO>
    where
        RO: Rate;

    fn split(self) -> (Self::InRate, Self::Partial);

    fn combine<RO>(rate: RO, partial: Self::Partial) -> Self::Output<RO>
    where
        RO: Rate;
}

impl<T, RI> ApplyRate for T
where
    T: ApplyRateChain<InRate = RI>,
    RI: Rate,
{
    type Output<R> = T::Output<ComposeRate<R, RI>> where R: Rate;

    fn apply_rate<R>(self, rate: R) -> Self::Output<R>
    where
        R: Rate,
    {
        let (in_rate, partial) = self.split();
        Self::combine(ComposeRate(rate, in_rate), partial)
    }
}

pub struct IdentityRate;

impl Rate for IdentityRate {
    fn eval(&self, t: f32) -> f32 {
        t
    }
}

pub struct ComposeRate<R0, R1>(R0, R1);

impl<R0, R1> Rate for ComposeRate<R0, R1>
where
    R0: Rate,
    R1: Rate,
{
    fn eval(&self, t: f32) -> f32 {
        self.0.eval(self.1.eval(t))
    }
}

pub struct ClampRate(Range<f32>);

impl Rate for ClampRate {
    fn eval(&self, t: f32) -> f32 {
        t.clamp(self.0.start, self.0.end)
    }
}

pub trait Clamp: Sized + ApplyRate {
    fn clamp(self, range: Range<f32>) -> Self::Output<ClampRate> {
        self.apply_rate(ClampRate(range))
    }
}

impl<T> Clamp for T where T: ApplyRate {}

pub struct SpeedRate(f32);

impl Rate for SpeedRate {
    fn eval(&self, t: f32) -> f32 {
        t * self.0
    }
}

pub trait Speed: Sized + ApplyRate {
    fn speed(self, speed: f32) -> Self::Output<SpeedRate> {
        self.apply_rate(SpeedRate(speed))
    }
}

impl<T> Speed for T where T: ApplyRate {}

pub struct SmoothRate;

impl Rate for SmoothRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

pub trait Smooth: Sized + ApplyRate {
    fn smooth(self) -> Self::Output<SmoothRate> {
        self.apply_rate(SmoothRate)
    }
}

impl<T> Smooth for T where T: ApplyRate {}

pub struct SmootherRate;

impl Rate for SmootherRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * t * (10.0 - t * (15.0 - 6.0 * t))
    }
}

pub trait Smoother: Sized + ApplyRate {
    fn smooth(self) -> Self::Output<SmootherRate> {
        self.apply_rate(SmootherRate)
    }
}

impl<T> Smoother for T where T: ApplyRate {}

// Refer to https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html
