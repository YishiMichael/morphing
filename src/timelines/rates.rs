use std::ops::Range;

pub(crate) trait WithRate<R>: Sized
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
}

pub trait ApplyRate<R>: WithRate<R>
where
    R: Rate,
{
    fn apply_rate<RI>(self, applied_rate: RI) -> Self::Output<Compose<RI, R>>
    where
        RI: Rate,
    {
        let (rate, partial) = self.split();
        Self::combine(Compose(applied_rate, rate), partial)
    }
    // {
    //     self.with_rate(|rate| Compose(applied_rate, rate))
    // }

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

impl<T, R> ApplyRate<R> for T
where
    T: WithRate<R>,
    R: Rate,
{
}

pub trait Rate {
    fn eval(&self, t: f32) -> f32;
}

pub(crate) struct Identity;

impl Rate for Identity {
    fn eval(&self, t: f32) -> f32 {
        t
    }
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
