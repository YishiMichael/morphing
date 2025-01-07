use super::super::mobjects::mobject::Mobject;

pub trait Timeline {}

trait DynamicTimelineContent {
    type Mobject: Mobject;
}

struct ContinuousTimelineContent<T>
where
    T: Mobject,
{
    mobject: T,
    diff: T::Diff,
}

impl<T> DynamicTimelineContent for ContinuousTimelineContent<T>
where
    T: Mobject,
{
    type Mobject = T;
}

struct DiscreteTimelineContent<T>
where
    T: Mobject,
{
    mobject: T,
    children: Vec<Box<dyn Timeline>>,
}

impl<T> DynamicTimelineContent for DiscreteTimelineContent<T>
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

struct DynamicTimeline<T, C, S>
where
    T: Mobject,
    C: DynamicTimelineContent<Mobject = T>,
    S: DynamicTimelineScale,
{
    timeline_content: C,
    timeline_scale: S,
}

impl<T, C, S> Timeline for DynamicTimeline<T, C, S>
where
    T: Mobject,
    C: DynamicTimelineContent<Mobject = T>,
    S: DynamicTimelineScale,
{
}

struct StaticTimeline<T>
where
    T: Mobject,
{
    mobject: T,
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
