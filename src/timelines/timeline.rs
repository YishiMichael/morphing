use std::marker::PhantomData;

use super::super::mobjects::mobject::Mobject;
use super::rates::Rate;
use super::rates::WithRate;

pub trait Timeline {}

trait DynamicTimelineContent {
    type Mobject: Mobject;
}

struct ContinuousTimelineContent<T, R>
where
    T: Mobject,
{
    mobject: T,
    diff: T::Diff,
    rate: R,
}

impl<T, R> DynamicTimelineContent for ContinuousTimelineContent<T, R>
where
    T: Mobject,
{
    type Mobject = T;
}

impl<T, R> WithRate<R> for ContinuousTimelineContent<T, R>
where
    T: Mobject,
    R: Rate,
{
    type Output<RO> = ContinuousTimelineContent<T, RO> where RO: Rate;

    fn with_rate<F, RO>(self, f: F) -> Self::Output<RO>
    where
        RO: super::rates::Rate,
        F: FnOnce(R) -> RO,
    {
        ContinuousTimelineContent {
            mobject: self.mobject,
            diff: self.diff,
            rate: f(self.rate),
        }
    }
}

struct DiscreteTimelineContent<T, R>
where
    T: Mobject,
{
    mobject: T,
    children: Vec<Box<dyn Timeline>>,
    rate: R,
}

impl<T, R> DynamicTimelineContent for DiscreteTimelineContent<T, R>
where
    T: Mobject,
{
    type Mobject = T;
}

trait DynamicTimelineScale {}

struct RelativeTimelineScale;

impl DynamicTimelineScale for RelativeTimelineScale {}

struct AbsoluteTimelineScale;

impl DynamicTimelineScale for AbsoluteTimelineScale {}

struct DynamicTimeline<C, S> {
    timeline_content: C,
    timeline_scale: S,
}

impl<C, S> Timeline for DynamicTimeline<C, S>
where
    C: DynamicTimelineContent,
    S: DynamicTimelineScale,
{
}

struct DynamicTimelineBuilder<T, S> {
    mobject: T,
    _phantom: PhantomData<S>,
}

struct StaticTimeline<T> {
    mobject: T,
}

impl<T> StaticTimeline<T>
where
    T: Mobject,
{
    pub fn animate(self) -> DynamicTimelineBuilder<T, RelativeTimelineScale> {
        DynamicTimelineBuilder {
            mobject: self.mobject,
            _phantom: PhantomData,
        }
    }

    pub fn animating(self) -> DynamicTimelineBuilder<T, AbsoluteTimelineScale> {
        DynamicTimelineBuilder {
            mobject: self.mobject,
            _phantom: PhantomData,
        }
    }
}

impl<T> Timeline for StaticTimeline<T> where T: Mobject {}

// pub trait StaticTimeline: Timeline {
//     type Relative: RelativeTimeline;
//     type Absolute: AbsoluteTimeline;
//     fn animate(self) -> (Timeli)
// }

// pub trait RelativeTimeline: Timeline {
// }

// pub trait AbsoluteTimeline: Timeline {
// }

// pub trait ContinuousRelativeTimeline: RelativeTimeline {fn update(&mut self, t: f32);}
// pub trait DiscreteRelativeTimeline: RelativeTimeline {fn update(&mut self, t: f32);}

// pub trait ContinuousAbsoluteTimeline: AbsoluteTimeline {
//     fn construct(&mut self);}

// pub trait DiscreteAbsoluteTimeline: AbsoluteTimeline {
//     fn construct(&mut self);}
