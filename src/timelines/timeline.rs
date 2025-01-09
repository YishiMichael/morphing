pub trait Timeline {}

pub mod steady {
    use super::super::super::mobjects::mobject::Mobject;
    use super::Timeline;

    pub struct SteadyTimeline<T> {
        pub mobject: T,
    }

    impl<T> Timeline for SteadyTimeline<T> where T: Mobject {}
}

pub mod dynamic {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::ApplyRate;
    use super::super::rates::IdentityRate;
    use super::super::rates::Rate;
    use super::steady::SteadyTimeline;
    use super::Timeline;

    pub trait DynamicTimelineContent {
        // type Mobject: Mobject;
    }

    pub trait DynamicTimelineMetric {}

    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {}

    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {}

    pub struct DynamicTimeline<C, M, R> {
        pub content: C,
        pub metric: M,
        pub rate: R,
    }

    impl<C, M, R> Timeline for DynamicTimeline<C, M, R>
    where
        C: DynamicTimelineContent,
        M: DynamicTimelineMetric,
        R: Rate,
    {
    }

    pub struct DynamicTimelineBuilder<T, M, R> {
        pub steady_mobject: SteadyTimeline<T>,
        pub metric: M,
        pub rate: R,
    }

    pub struct DynamicTimelineBuilderPartial<T, M> {
        steady_mobject: SteadyTimeline<T>,
        metric: M,
    }

    impl<T, M, R> ApplyRate<R> for DynamicTimelineBuilder<T, M, R>
    where
        T: Mobject,
        M: DynamicTimelineMetric,
        R: Rate,
    {
        type Partial = DynamicTimelineBuilderPartial<T, M>;
        type Output<RO> = DynamicTimelineBuilder<T, M, RO>
        where
            RO: Rate;

        fn split(self) -> (R, Self::Partial) {
            (
                self.rate,
                DynamicTimelineBuilderPartial {
                    steady_mobject: self.steady_mobject,
                    metric: self.metric,
                },
            )
        }

        fn combine<RO>(rate: RO, partial: Self::Partial) -> Self::Output<RO>
        where
            RO: Rate,
        {
            DynamicTimelineBuilder {
                steady_mobject: partial.steady_mobject,
                metric: partial.metric,
                rate,
            }
        }
    }

    impl<T> SteadyTimeline<T>
    where
        T: Mobject,
    {
        pub fn animate(self) -> DynamicTimelineBuilder<T, RelativeTimelineMetric, IdentityRate> {
            DynamicTimelineBuilder {
                steady_mobject: self,
                metric: RelativeTimelineMetric,
                rate: IdentityRate,
            }
        }

        pub fn animating(self) -> DynamicTimelineBuilder<T, AbsoluteTimelineMetric, IdentityRate> {
            DynamicTimelineBuilder {
                steady_mobject: self,
                metric: AbsoluteTimelineMetric,
                rate: IdentityRate,
            }
        }
    }
}

pub mod action {
    // use super::super::super::components::interpolate::Interpolate;
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::rates::Rate;
    use super::dynamic::DynamicTimeline;
    use super::dynamic::DynamicTimelineBuilder;
    use super::dynamic::DynamicTimelineContent;
    use super::dynamic::DynamicTimelineMetric;

    pub struct ActionTimelineContent<T>
    where
        T: Mobject,
    {
        source_mobject: T,
        target_mobject: T,
    }

    impl<T, M, R> DynamicTimeline<ActionTimelineContent<T>, M, R>
    where
        T: Mobject,
        M: DynamicTimelineMetric,
        R: Rate,
    {
        pub fn act<A>(mut self, act: A) -> Self
        where
            A: Act<T>,
        {
            act.act(&mut self.content.target_mobject);
            self
        }
    }

    impl<T, M, R> DynamicTimelineBuilder<T, M, R>
    where
        T: Mobject,
        M: DynamicTimelineMetric,
        R: Rate,
    {
        pub fn act<A>(self, act: A) -> DynamicTimeline<ActionTimelineContent<T>, M, R>
        where
            A: Act<T>,
        {
            let source_mobject = self.steady_mobject.mobject;
            let target_mobject = source_mobject.clone();
            let content = ActionTimelineContent {
                source_mobject,
                target_mobject,
            };
            DynamicTimeline {
                content: content,
                metric: self.metric,
                rate: self.rate,
            }
            .act(act)
        }
    }

    impl<T> DynamicTimelineContent for ActionTimelineContent<T> where T: Mobject {}
}

pub mod continuous {
    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::DynamicTimelineContent;

    pub struct ContinuousTimelineContent<T>
    where
        T: Mobject,
    {
        mobject: T,
    }

    impl<T> DynamicTimelineContent for ContinuousTimelineContent<T> where T: Mobject {}
}

pub mod discrete {
    use super::dynamic::DynamicTimelineContent;
    use super::Timeline;

    pub struct DiscreteTimelineContent {
        children: Vec<Box<dyn Timeline>>,
    }

    impl DynamicTimelineContent for DiscreteTimelineContent {}
}

// pub trait SteadyTimeline: TimelineContent {
//     type Relative: RelativeTimeline;
//     type Absolute: AbsoluteTimeline;
//     fn animate(self) -> (Timeli)
// }

// pub trait RelativeTimeline: TimelineContent {
// }

// pub trait AbsoluteTimeline: TimelineContent {
// }

// pub trait ContinuousRelativeTimeline: RelativeTimeline {fn update(&mut self, t: f32);}
// pub trait DiscreteRelativeTimeline: RelativeTimeline {fn update(&mut self, t: f32);}

// pub trait ContinuousAbsoluteTimeline: AbsoluteTimeline {
//     fn construct(&mut self);}

// pub trait DiscreteAbsoluteTimeline: AbsoluteTimeline {
//     fn construct(&mut self);}
