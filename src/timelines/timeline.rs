pub trait Timeline {}

pub mod r#static {
    use super::super::super::mobjects::mobject::Mobject;
    use super::Timeline;

    pub(crate) struct StaticTimeline<T> {
        mobject: T,
    }

    impl<T> Timeline for StaticTimeline<T> where T: Mobject {}
}

pub mod dynamic {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::Identity;
    use super::super::rates::Rate;
    use super::super::rates::WithRate;
    use super::r#static::StaticTimeline;
    use super::Timeline;

    pub(crate) trait DynamicTimelineNode {
        // type Mobject: Mobject;
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

    impl<N, M, R> Timeline for DynamicTimeline<N, M, R>
    where
        N: DynamicTimelineNode,
        M: DynamicTimelineMetric,
        R: Rate,
    {
    }

    struct DynamicTimelineBuilder<T, M, R> {
        static_mobject: StaticTimeline<T>,
        metric: M,
        rate: R,
    }

    struct DynamicTimelineBuilderPartial<T, M> {
        static_mobject: StaticTimeline<T>,
        metric: M,
    }

    impl<T, M, R> WithRate<R> for DynamicTimelineBuilder<T, M, R>
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
                    static_mobject: self.static_mobject,
                    metric: self.metric,
                },
            )
        }

        fn combine<RO>(rate: RO, partial: Self::Partial) -> Self::Output<RO>
        where
            RO: Rate,
        {
            DynamicTimelineBuilder {
                static_mobject: partial.static_mobject,
                metric: partial.metric,
                rate,
            }
        }
    }

    impl<T> StaticTimeline<T>
    where
        T: Mobject,
    {
        pub fn animate(self) -> DynamicTimelineBuilder<T, RelativeTimelineMetric, Identity> {
            DynamicTimelineBuilder {
                static_mobject: self,
                metric: RelativeTimelineMetric,
                rate: Identity,
            }
        }

        pub fn animating(self) -> DynamicTimelineBuilder<T, AbsoluteTimelineMetric, Identity> {
            DynamicTimelineBuilder {
                static_mobject: self,
                metric: AbsoluteTimelineMetric,
                rate: Identity,
            }
        }
    }
}

pub mod action {
    use super::super::super::components::interpolate::Interpolate;
    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::DynamicTimelineNode;

    pub trait Act<T>
    where
        T: Mobject + Interpolate,
    {
        fn act(&self, mobject: &mut T);
    }

    struct Node<T>
    where
        T: Mobject + Interpolate,
    {
        source_mobject: T,
        target_mobject: T,
    }

    impl<T> DynamicTimelineNode for Node<T> where T: Mobject + Interpolate {}
}

pub mod continuous {
    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::DynamicTimelineNode;

    pub trait Update<T>
    where
        T: Mobject,
    {
        fn update(self, mobject: &T, alpha: f32);
    }

    struct Node<T>
    where
        T: Mobject,
    {
        mobject: T,
    }

    impl<T> DynamicTimelineNode for Node<T> where T: Mobject {}
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::DynamicTimelineNode;
    use super::r#static::StaticTimeline;
    use super::Timeline;

    pub trait Construct<T>
    where
        T: Mobject,
    {
        fn construct(self, static_mobject: StaticTimeline<T>);
    }

    struct Node {
        children: Vec<Box<dyn Timeline>>,
    }

    impl DynamicTimelineNode for Node {}
}

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
