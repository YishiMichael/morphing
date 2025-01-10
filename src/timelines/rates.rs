use std::ops::Range;

pub trait ApplyRate<R>: Sized
where
    R: Rate,
{
    type Partial;
    type Output<RO>
    where
        RO: Rate;

    fn split(self) -> (R, Self::Partial);

    fn combine<RO>(rate: RO, partial: Self::Partial) -> Self::Output<RO>
    where
        RO: Rate;

    fn apply_rate<RI>(self, applied_rate: RI) -> Self::Output<ComposeRate<RI, R>>
    where
        RI: Rate,
    {
        let (rate, partial) = self.split();
        Self::combine(ComposeRate(applied_rate, rate), partial)
    }
}

pub trait Rate: 'static {
    fn eval(&self, t: f32) -> f32;
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

pub trait Clamp<R>: ApplyRate<R>
where
    R: Rate,
{
    fn clamp(self, range: Range<f32>) -> Self::Output<ComposeRate<ClampRate, R>> {
        self.apply_rate(ClampRate(range))
    }
}

impl<T, R> Clamp<R> for T
where
    T: ApplyRate<R>,
    R: Rate,
{
}

pub struct SpeedRate(f32);

impl Rate for SpeedRate {
    fn eval(&self, t: f32) -> f32 {
        t * self.0
    }
}

pub trait Speed<R>: ApplyRate<R>
where
    R: Rate,
{
    fn speed(self, speed: f32) -> Self::Output<ComposeRate<SpeedRate, R>> {
        self.apply_rate(SpeedRate(speed))
    }
}

impl<T, R> Speed<R> for T
where
    T: ApplyRate<R>,
    R: Rate,
{
}

pub struct SmoothRate;

impl Rate for SmoothRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

pub trait Smooth<R>: ApplyRate<R>
where
    R: Rate,
{
    fn smooth(self) -> Self::Output<ComposeRate<SmoothRate, R>> {
        self.apply_rate(SmoothRate)
    }
}

impl<T, R> Smooth<R> for T
where
    T: ApplyRate<R>,
    R: Rate,
{
}

pub struct SmootherRate;

impl Rate for SmootherRate {
    fn eval(&self, t: f32) -> f32 {
        t * t * t * (10.0 - t * (15.0 - 6.0 * t))
    }
}

pub trait Smoother<R>: ApplyRate<R>
where
    R: Rate,
{
    fn smooth(self) -> Self::Output<ComposeRate<SmootherRate, R>> {
        self.apply_rate(SmootherRate)
    }
}

impl<T, R> Smoother<R> for T
where
    T: ApplyRate<R>,
    R: Rate,
{
}

// Refer to https://docs.rs/interpolation/latest/src/interpolation/ease.rs.html
