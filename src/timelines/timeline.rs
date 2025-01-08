use super::super::mobjects::mobject::Mobject;
use super::rates::Rate;
use super::rates::WithRate;

pub trait Timeline {}

trait DynamicTimelineNode {
    // type Mobject: Mobject;
}

struct ContinuousTimelineNode<T>
where
    T: Mobject,
{
    mobject: T,
    diff: T::Diff,
}

impl<T> DynamicTimelineNode for ContinuousTimelineNode<T>
where
    T: Mobject,
{
    // type Mobject = T;
}

struct DiscreteTimelineNode<T>
where
    T: Mobject,
{
    mobject: T,
    children: Vec<Box<dyn Timeline>>,
}

impl<T> DynamicTimelineNode for DiscreteTimelineNode<T>
where
    T: Mobject,
{
    // type Mobject = T;
}

trait DynamicTimelineMetric {}

struct RelativeTimelineMetric;

impl DynamicTimelineMetric for RelativeTimelineMetric {}

struct AbsoluteTimelineMetric;

impl DynamicTimelineMetric for AbsoluteTimelineMetric {}

struct DynamicTimeline<N, M, R> {
    node: N,
    metric: M,
    rate: R,
}

impl<N, M, R> WithRate<R> for DynamicTimeline<N, M, R>
where
    N: DynamicTimelineNode,
    M: DynamicTimelineMetric,
    R: Rate,
{
    type Output<RO> = DynamicTimeline<N, M, RO>
    where
        RO: Rate;

    fn with_rate<F, RO>(self, f: F) -> Self::Output<RO>
    where
        RO: Rate,
        F: FnOnce(R) -> RO,
    {
        DynamicTimeline {
            node: self.node,
            metric: self.metric,
            rate: f(self.rate),
        }
    }
}

impl<N, M, R> Timeline for DynamicTimeline<N, M, R>
where
    N: DynamicTimelineNode,
    M: DynamicTimelineMetric,
    R: Rate,
{
}

struct DynamicTimelineBuilder<T, M> {
    mobject: T,
    metric: M,
}

struct StaticTimeline<T> {
    mobject: T,
}

impl<T> StaticTimeline<T>
where
    T: Mobject,
{
    pub fn animate(self) -> DynamicTimelineBuilder<T, RelativeTimelineMetric> {
        DynamicTimelineBuilder {
            mobject: self.mobject,
            metric: RelativeTimelineMetric,
        }
    }

    pub fn animating(self) -> DynamicTimelineBuilder<T, AbsoluteTimelineMetric> {
        DynamicTimelineBuilder {
            mobject: self.mobject,
            metric: AbsoluteTimelineMetric,
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
