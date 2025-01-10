use std::any::Any;
use std::marker::PhantomData;
use std::ops::Range;

use super::super::mobjects::mobject::Mobject;

pub trait Timeline: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_mut(&mut self) -> &mut dyn Any;
    fn presentation(self, time_interval: Range<f32>) -> Box<dyn Present>;
}

pub trait Present: 'static {
    fn present(&self, time: f32);
}

// struct Updater {
//     time_interval: Range<f32>,
//     rate: Box<dyn Rate>,
// }

// pub struct PresentationCollection {
//     timestamps: Vec<f32>,
//     presentations: Vec<(Range<usize>, Box<dyn Present>)>,
// }

// impl PresentationCollection {
//     fn embed(&mut self, timestamp_index_interval: Range<usize>, other: Self) {
//         todo!()
//     }
// }

struct SupervisorInner {
    timestamps: Vec<f32>,
    timelines: Vec<(Range<usize>, Box<dyn Timeline>)>,
}

impl SupervisorInner {
    fn new() -> Self {
        Self {
            timestamps: vec![0.0],
            timelines: Vec::new(),
        }
    }

    fn wait(&mut self, time: f32) {
        self.timestamps.push(self.timestamps.last().unwrap() + time);
    }

    fn new_timeline<T>(&mut self, timeline: T) -> usize
    where
        T: Timeline,
    {
        self.timelines
            .push((self.timestamps.len() - 1..0, Box::new(timeline)));
        self.timelines.len() - 1
    }

    fn drop_timeline(&mut self, timeline_index: usize) {
        self.timelines[timeline_index].0.end = self.timelines.len()
    }

    fn get_timeline<T>(&self, timeline_index: usize) -> &T
    where
        T: Timeline,
    {
        self.timelines[timeline_index]
            .1
            .as_ref()
            .as_any()
            .downcast_ref::<T>()
            .unwrap()
    }

    fn get_timeline_mut<T>(&mut self, timeline_index: usize) -> &mut T
    where
        T: Timeline,
    {
        self.timelines[timeline_index]
            .1
            .as_mut()
            .as_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}

pub struct Supervisor(parking_lot::Mutex<SupervisorInner>);

impl Supervisor {
    fn new() -> Self {
        Self(parking_lot::Mutex::new(SupervisorInner::new()))
    }

    fn spawn_timeline<T>(&self, timeline: T) -> Alive<T>
    where
        T: Timeline,
    {
        Alive::new(self, timeline)
    }

    pub fn spawn<M>(&self, mobject: M) -> Alive<steady::SteadyTimeline<M>>
    where
        M: Mobject,
    {
        self.spawn_timeline(steady::SteadyTimeline { mobject })
    }

    pub fn wait(&self, time: f32) {
        self.0.lock().wait(time);
        // self.timestamps
        //     .lock()
        //     .push(self.timestamps.lock().last().unwrap() + time);
        // self.timestamps.push(self.current_timestamp);
        // self.current_timestamp += time;
        // self.timestamp_index += 1;
    }

    fn new_timeline<T>(&self, timeline: T) -> usize
    where
        T: Timeline,
    {
        self.0.lock().new_timeline(timeline)
    }

    fn drop_timeline(&self, timeline_index: usize) {
        self.0.lock().drop_timeline(timeline_index);
    }

    fn call_timeline<T, F, O>(&self, timeline_index: usize, f: F) -> O
    where
        T: Timeline,
        F: FnOnce(&T) -> O,
    {
        f(self.0.lock().get_timeline(timeline_index))
    }

    fn call_timeline_mut<T, F, O>(&self, timeline_index: usize, f: F) -> O
    where
        T: Timeline,
        F: FnOnce(&mut T) -> O,
    {
        f(self.0.lock().get_timeline_mut(timeline_index))
    }
}

pub struct Alive<'a, T>
where
    T: Timeline,
{
    // spawn_timestamp_index: usize,
    timeline_index: usize,
    supervisor: &'a Supervisor,
    phantom: PhantomData<T>,
    // timeline: T,
}

impl<'a, T> Alive<'a, T>
where
    T: Timeline,
{
    fn new(supervisor: &'a Supervisor, timeline: T) -> Self {
        // supervisor.timelines.lock().push((
        //     supervisor.timestamps.lock().len() - 1..0,
        //     Box::new(timeline),
        // ));
        Self {
            timeline_index: supervisor.new_timeline(timeline),
            supervisor,
            phantom: PhantomData,
        }
    }

    fn call_timeline<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&T) -> O,
    {
        // <dyn Timeline>::downcast_ref::<T>(self.supervisor.timelines[self.timeline_index].1.as_ref())
        //     .unwrap()
        self.supervisor.call_timeline(self.timeline_index, f)
    }

    fn call_timeline_mut<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut T) -> O,
    {
        // <dyn Timeline>::downcast_ref::<T>(self.supervisor.timelines[self.timeline_index].1.as_ref())
        //     .unwrap()
        self.supervisor.call_timeline_mut(self.timeline_index, f)
    }

    // fn get_mut(&mut self) -> &mut T {
    //     // <dyn Timeline>::downcast_ref::<T>(self.supervisor.timelines[self.timeline_index].1.as_ref())
    //     //     .unwrap()
    //     self.supervisor.timelines.lock()[self.timeline_index]
    //         .1
    //         .as_mut()
    //         .as_mut()
    //         .downcast_mut::<T>()
    //         .unwrap()
    // }
}

impl<'a, T> Drop for Alive<'a, T>
where
    T: Timeline,
{
    fn drop(&mut self) {
        // self.supervisor.ranged_timelines.presentation_collection.embed(
        //     self.spawn_timestamp_index..self.supervisor.timestamp_index,
        //     self.timeline
        //         .collect_presentations(self.spawn_timestamp..self.supervisor.current_timestamp),
        // );
        // self.supervisor.timelines.lock()[self.timeline_index].0.end =
        //     self.supervisor.timelines.lock().len();
        self.supervisor.drop_timeline(self.timeline_index);
    }
}

pub mod steady {
    use std::any::Any;
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    // use super::Present;
    // use super::PresentationCollection;
    use super::Alive;
    use super::Present;
    use super::Timeline;

    pub struct SteadyTimeline<T> {
        pub mobject: T,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn presentation(self, _: Range<f32>) -> Box<dyn Present> {
            Box::new(SteadyTimelinePresentation {
                mobject: self.mobject,
            })
        }
    }

    impl<'a, M, A> ApplyAct<M, A> for Alive<'a, SteadyTimeline<M>>
    where
        M: Mobject,
        A: Act<M>,
    {
        type Output = Self;

        fn apply_act(self, act: A) -> Self::Output {
            let mut mobject = self.call_timeline(|timeline| timeline.mobject.clone());
            act.act(&mut mobject);
            self.supervisor.spawn_timeline(SteadyTimeline { mobject })
        }
    }

    struct SteadyTimelinePresentation<T> {
        mobject: T,
    }

    impl<T> Present for SteadyTimelinePresentation<T>
    where
        T: Mobject,
    {
        fn present(&self, _: f32) {
            self.mobject.render();
        }
    }
}

pub mod dynamic {
    use std::any::Any;
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::ApplyRate;
    use super::super::rates::IdentityRate;
    use super::super::rates::Rate;
    use super::steady::SteadyTimeline;
    use super::Alive;
    use super::Present;
    use super::Timeline;

    pub trait DynamicTimelineContent: 'static {
        type ContentPresentation: Present;

        fn content_presentation(self) -> Self::ContentPresentation;
    }

    pub trait DynamicTimelineMetric: 'static {
        fn metric_scalar(&self, time_interval: Range<f32>) -> f32;
    }

    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {
        fn metric_scalar(&self, time_interval: Range<f32>) -> f32 {
            1.0 / (time_interval.end - time_interval.start)
        }
    }

    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {
        fn metric_scalar(&self, _: Range<f32>) -> f32 {
            1.0
        }
    }

    pub struct DynamicTimeline<CO, ME, R> {
        pub content: CO,
        pub metric: ME,
        pub rate: R,
    }

    impl<CO, ME, R> Timeline for DynamicTimeline<CO, ME, R>
    where
        CO: DynamicTimelineContent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn presentation(self, time_interval: Range<f32>) -> Box<dyn Present> {
            Box::new(DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(),
                rate: self.rate,
                start_time: time_interval.start,
                metric_scalar: self.metric.metric_scalar(time_interval),
            })
        }
    }

    struct DynamicTimelinePresentation<P, R> {
        content_presentation: P,
        rate: R,
        start_time: f32,
        metric_scalar: f32,
    }

    impl<P, R> Present for DynamicTimelinePresentation<P, R>
    where
        P: Present,
        R: Rate,
    {
        fn present(&self, time: f32) {
            self.content_presentation.present(
                self.rate
                    .eval(self.metric_scalar * (time - self.start_time)),
            );
        }
    }

    pub struct DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
    {
        pub steady_mobject: Alive<'a, SteadyTimeline<M>>,
        pub metric: ME,
        pub rate: R,
    }

    pub struct DynamicTimelineBuilderPartial<'a, M, ME>
    where
        M: Mobject,
    {
        steady_mobject: Alive<'a, SteadyTimeline<M>>,
        metric: ME,
    }

    impl<'a, M, ME, R> ApplyRate<R> for DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Partial = DynamicTimelineBuilderPartial<'a, M, ME>;
        type Output<RO> = DynamicTimelineBuilder<'a, M, ME, RO>
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

    impl<'a, M> Alive<'a, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        pub fn animate(
            self,
        ) -> DynamicTimelineBuilder<'a, M, RelativeTimelineMetric, IdentityRate> {
            DynamicTimelineBuilder {
                steady_mobject: self,
                metric: RelativeTimelineMetric,
                rate: IdentityRate,
            }
        }

        pub fn animating(
            self,
        ) -> DynamicTimelineBuilder<'a, M, AbsoluteTimelineMetric, IdentityRate> {
            DynamicTimelineBuilder {
                steady_mobject: self,
                metric: AbsoluteTimelineMetric,
                rate: IdentityRate,
            }
        }
    }
}

pub mod action {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    use super::super::rates::Rate;
    use super::dynamic::DynamicTimeline;
    use super::dynamic::DynamicTimelineBuilder;
    use super::dynamic::DynamicTimelineContent;
    use super::dynamic::DynamicTimelineMetric;
    use super::Alive;
    use super::Present;

    pub struct ActionTimelineContent<M>
    where
        M: Mobject,
    {
        source_mobject: M,
        target_mobject: M,
    }

    impl<M, A, ME, R> ApplyAct<M, A> for Alive<'_, DynamicTimeline<ActionTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        A: Act<M>,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output = Self;

        fn apply_act(self, act: A) -> Self::Output {
            self.call_timeline_mut(|timeline| act.act(&mut timeline.content.target_mobject));
            // self.supervisor.spawn(mobject)
            self
        }
    }

    // impl<M, ME, R> DynamicTimeline<ActionTimelineContent<M>, ME, R>
    // where
    //     M: Mobject,
    //     ME: DynamicTimelineMetric,
    //     R: Rate,
    // {
    //     pub fn act<A>(mut self, act: A) -> Self
    //     where
    //         A: Act<M>,
    //     {
    //         act.act(&mut self.content.target_mobject);
    //         self
    //     }
    // }

    // impl<A, M> ApplyAct<A, M>

    impl<'a, M, A, ME, R> ApplyAct<M, A> for DynamicTimelineBuilder<'a, M, ME, R>
    where
        M: Mobject,
        A: Act<M>,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output = Alive<'a, DynamicTimeline<ActionTimelineContent<M>, ME, R>>;

        fn apply_act(self, act: A) -> Self::Output
        where
            A: Act<M>,
        {
            let source_mobject = self
                .steady_mobject
                .call_timeline(|timeline| timeline.mobject.clone());
            let mut target_mobject = source_mobject.clone();
            act.act(&mut target_mobject);
            let content = ActionTimelineContent {
                source_mobject,
                target_mobject,
            };
            self.steady_mobject
                .supervisor
                .spawn_timeline(DynamicTimeline {
                    content,
                    metric: self.metric,
                    rate: self.rate,
                })
        }
    }

    impl<M> DynamicTimelineContent for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = ActionTimelineContentPresentation<M>;

        fn content_presentation(self) -> Self::ContentPresentation {
            todo!()
        }
    }

    struct ActionTimelineContentPresentation<M> {}

    impl<M> Present for ActionTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        fn present(&self, time: f32) {
            todo!()
        }
    }
}

pub mod continuous {
    use crate::timelines::update::Update;

    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::DynamicTimelineContent;
    use super::Present;

    pub struct ContinuousTimelineContent<M, U> {
        mobject: M,
        update: U,
    }

    impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type ContentPresentation = ContinuousTimelineContentPresentation<M, U>;

        fn content_presentation(self) -> Self::ContentPresentation {
            ContinuousTimelineContentPresentation {
                mobject: self.mobject,
                update: self.update,
            }
        }
    }

    struct ContinuousTimelineContentPresentation<M, U> {
        mobject: M,
        update: U,
    }

    impl<M, U> Present for ContinuousTimelineContentPresentation<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        fn present(&self, time: f32) {
            self.update.update(&self.mobject, time).render();
        }
    }
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::construct::Construct;
    use super::dynamic::DynamicTimelineContent;

    pub struct DiscreteTimelineContent<M, C> {
        mobject: M,
        construct: C,
    }

    impl<M, C> DynamicTimelineContent for DiscreteTimelineContent<M, C>
    where
        M: Mobject,
        C: Construct<M>,
    {
        type ContentPresentation = DiscreteTimelineContentPresentation<M, C>;

        fn content_presentation(self) -> Self::ContentPresentation {
            DiscreteTimelineContentPresentation {
                mobject: self.mobject,
                // construct: self.construct,
            }
        }
    }
}
