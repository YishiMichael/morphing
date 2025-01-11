use std::ops::Range;

use super::super::mobjects::mobject::Mobject;

pub trait Timeline: 'static {
    type Presentation: Present;

    fn presentation(self) -> Self::Presentation;
}

pub trait Present: 'static {
    fn present(&self, time: f32, time_interval: Range<f32>);
}

// struct Updater {
//     time_interval: Range<f32>,
//     rate: Box<dyn Rate>,
// }

// pub struct PresentationCollection {
//     timestamps: Vec<f32>,
//     timelines: Vec<(Range<usize>, Box<dyn Present>)>,
// }

// impl PresentationCollection {
//     fn embed(&mut self, timestamp_index_interval: Range<usize>, other: Self) {
//         todo!()
//     }
// }

// struct SupervisorData {
//     timestamps: Vec<f32>,
//     timelines: Vec<(Range<usize>, Box<dyn Timeline>)>,
// }

// impl SupervisorData {
//     fn new() -> Self {
//         Self {
//             timestamps: vec![0.0],
//             timelines: Vec::new(),
//         }
//     }

//     fn wait(&mut self, time: f32) {
//         self.timestamps.push(self.timestamps.last().unwrap() + time);
//     }

//     fn new_timeline<T>(&mut self, timeline: T) -> usize
//     where
//         T: Timeline,
//     {
//         self.timelines
//             .push((self.timestamps.len() - 1..0, Box::new(timeline)));
//         self.timelines.len() - 1
//     }

//     fn drop_timeline(&mut self, timeline_index: usize) {
//         self.timelines[timeline_index].0.end = self.timelines.len()
//     }

//     fn get_timeline<T>(&self, timeline_index: usize) -> &T
//     where
//         T: Timeline,
//     {
//         self.timelines[timeline_index]
//             .1
//             .as_ref()
//             .as_any()
//             .downcast_ref::<T>()
//             .unwrap()
//     }

//     fn get_timeline_mut<T>(&mut self, timeline_index: usize) -> &mut T
//     where
//         T: Timeline,
//     {
//         self.timelines[timeline_index]
//             .1
//             .as_mut()
//             .as_mut()
//             .downcast_mut::<T>()
//             .unwrap()
//     }
// }

struct SupervisorData {
    timestamps: Vec<f32>,
    presentations: Vec<(Range<usize>, Box<dyn Present>)>,
}

impl SupervisorData {
    fn supervisor_present(&self, time: f32) {
        let timestamp_index = self
            .timestamps
            .partition_point(|&timestamp| timestamp < time);
        for (timestamp_index_range, present) in &self.presentations {
            if timestamp_index_range.contains(&timestamp_index) {
                present.present(
                    time,
                    self.timestamps[timestamp_index_range.start]
                        ..self.timestamps[timestamp_index_range.end],
                );
            }
        }
    }
}

pub struct Supervisor(parking_lot::Mutex<SupervisorData>);

impl Supervisor {
    fn new() -> Self {
        Self(parking_lot::Mutex::new(SupervisorData {
            timestamps: vec![0.0],
            presentations: Vec::new(),
        }))
    }

    // fn spawn_timeline<T>(&self, timeline: T) -> Alive<T>
    // where
    //     T: Timeline,
    // {
    //     let guard = self.0.lock();
    //     Alive {
    //         spawn_timestamp_index: guard.timestamps.len() - 1,
    //         timeline,
    //         supervisor: self,
    //     }
    // }

    // fn archive_timeline<T>(&self, alive_timeline: Alive<T>)
    // where
    //     T: Timeline,
    // {
    //     let mut guard = self.0.lock();
    //     let timestamp_index_interval =
    //         alive_timeline.spawn_timestamp_index..guard.timestamps.len() - 1;
    //     if !timestamp_index_interval.is_empty() {
    //         let time_interval = guard.timestamps[timestamp_index_interval.start]
    //             ..guard.timestamps[timestamp_index_interval.end];
    //         guard.timelines.push((
    //             timestamp_index_interval,
    //             alive_timeline.timeline.presentation(time_interval),
    //         ));
    //     }
    // }

    pub fn spawn<M>(&self, mobject: M) -> Alive<steady::SteadyTimeline<M>>
    where
        M: Mobject,
    {
        Alive::new(steady::SteadyTimeline { mobject }, self)
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(!delta_time.is_sign_negative());
        let mut guard = self.0.lock();
        let time = guard.timestamps.last().unwrap() + delta_time;
        guard.timestamps.push(time);
        // self.timestamps
        //     .lock()
        //     .push(self.timestamps.lock().last().unwrap() + time);
        // self.timestamps.push(self.current_timestamp);
        // self.current_timestamp += time;
        // self.timestamp_index += 1;
    }

    // fn new_timeline<T>(&self, timeline: T) -> usize
    // where
    //     T: Timeline,
    // {
    //     self.timelines
    //         .push((self.timestamps.len() - 1..0, Box::new(timeline)));
    //     self.timelines.len() - 1
    // }

    // fn del_timeline(&self, timeline_index: usize) {
    //     self.timelines[timeline_index].0.end = self.timelines.len()
    // }

    // fn call_timeline<T, F, O>(&self, timeline_index: usize, f: F) -> O
    // where
    //     T: Timeline,
    //     F: FnOnce(Range<f32>, &T) -> O,
    // {
    //     let guard = self.0.lock();
    //     f(
    //         guard.get_time_interval(timeline_index),
    //         guard.get_timeline(timeline_index),
    //     )
    // }

    // fn call_timeline_mut<T, F, O>(&self, timeline_index: usize, f: F) -> O
    // where
    //     T: Timeline,
    //     F: FnOnce(Range<f32>, &mut T) -> O,
    // {
    //     let mut guard = self.0.lock();
    //     f(
    //         guard.get_time_interval(timeline_index),
    //         guard.get_timeline_mut(timeline_index),
    //     )
    // }
}

pub struct Alive<'a, T>
where
    T: Timeline,
{
    spawn_timestamp_index: usize,
    timeline: T,
    // timeline_index: usize,
    supervisor: &'a Supervisor,
    // phantom: PhantomData<T>,
}

impl<'a, T> Alive<'a, T>
where
    T: Timeline,
{
    fn new(timeline: T, supervisor: &'a Supervisor) -> Self {
        // supervisor.timelines.lock().push((
        //     supervisor.timestamps.lock().len() - 1..0,
        //     Box::new(timeline),
        // ));
        let guard = supervisor.0.lock();
        Self {
            spawn_timestamp_index: guard.timestamps.len() - 1,
            timeline,
            supervisor,
            // phantom: PhantomData,
        }
    }

    fn archive<F, O>(self, f: F) -> O
    where
        F: FnOnce(&T::Presentation, &'a Supervisor, Range<f32>) -> O,
    {
        let mut guard = self.supervisor.0.lock();
        let timestamp_index_interval = self.spawn_timestamp_index..guard.timestamps.len() - 1;
        // if !timestamp_index_interval.is_empty() {
        // let time_interval = ;
        let presentation = self.timeline.presentation();
        let output = f(
            &presentation,
            self.supervisor,
            guard.timestamps[timestamp_index_interval.start]
                ..guard.timestamps[timestamp_index_interval.end],
        );
        guard
            .presentations
            .push((timestamp_index_interval, Box::new(presentation)));
        output
        // }
    }

    // fn renew<F, TO>(self, f: F) -> Alive<'a, TO>
    // where
    //     F: FnOnce(&T::Presentation, Range<f32>) -> TO,
    //     TO: Timeline,
    // {
    //     self.archive(|supervisor, presentation, time_interval| {
    //         Alive::new(supervisor, f(presentation, time_interval))
    //     })
    // }

    // fn call_timeline<F, O>(&self, f: F) -> O
    // where
    //     F: FnOnce(Range<f32>, &T) -> O,
    // {
    //     // <dyn Timeline>::downcast_ref::<T>(self.supervisor.timelines[self.timeline_index].1.as_ref())
    //     //     .unwrap()
    //     self.supervisor.call_timeline(self.timeline_index, f)
    // }

    // fn call_timeline_mut<F, O>(&self, f: F) -> O
    // where
    //     F: FnOnce(Range<f32>, &mut T) -> O,
    // {
    //     // <dyn Timeline>::downcast_ref::<T>(self.supervisor.timelines[self.timeline_index].1.as_ref())
    //     //     .unwrap()
    //     self.supervisor.call_timeline_mut(self.timeline_index, f)
    // }

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

// impl<'a, T> Drop for Alive<'a, T>
// where
//     T: Timeline,
// {
//     fn drop(&mut self) {
//         // self.supervisor.ranged_timelines.presentation_collection.embed(
//         //     self.spawn_timestamp_index..self.supervisor.timestamp_index,
//         //     self.timeline
//         //         .collect_timelines(self.spawn_timestamp..self.supervisor.current_timestamp),
//         // );
//         // self.supervisor.timelines.lock()[self.timeline_index].0.end =
//         //     self.supervisor.timelines.lock().len();
//         self.supervisor.drop_timeline(self.timeline_index);
//     }
// }

pub mod steady {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    // use super::Present;
    // use super::PresentationCollection;
    use super::Alive;
    use super::Present;
    // use super::Present;
    use super::Timeline;

    pub struct SteadyTimeline<M> {
        pub mobject: M,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        type Presentation = Self;

        fn presentation(self) -> Self::Presentation {
            self
        }
    }

    impl<M> Present for SteadyTimeline<M>
    where
        M: Mobject,
    {
        fn present(&self, _time: f32, _time_interval: Range<f32>) {
            self.mobject.render();
        }
    }

    impl<M> Alive<'_, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        fn destroy(self) {
            self.archive(|_, _, _| ())
        }
    }

    impl<'a, M, A> ApplyAct<M, A> for Alive<'_, SteadyTimeline<M>>
    where
        M: Mobject,
        A: Act<M>,
    {
        type Output = Self;

        fn apply_act(self, act: A) -> Self::Output {
            self.archive(|SteadyTimeline { mobject }, supervisor, _| {
                Alive::new(
                    SteadyTimeline {
                        mobject: mobject.apply_diff(act.act(mobject)),
                    },
                    supervisor,
                )
            })
        }
    }
}

pub mod dynamic {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::ApplyRate;
    use super::super::rates::IdentityRate;
    use super::super::rates::Rate;
    use super::steady::SteadyTimeline;
    use super::Alive;
    use super::Present;
    // use super::Present;
    use super::Timeline;

    pub trait ContentPresent: 'static {
        fn content_present(&self, time: f32);
    }

    pub trait Collapse {
        type Output: Mobject;

        fn collapse(&self, time: f32) -> Self::Output;
    }

    pub trait DynamicTimelineContent: 'static {
        type ContentPresentation: ContentPresent + Collapse;

        fn content_presentation(self) -> Self::ContentPresentation;

        // fn content_present(&self, time: f32);
    }

    pub trait DynamicTimelineMetric: 'static {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32;
    }

    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            (time - time_interval.start) / (time_interval.end - time_interval.start)
        }
    }

    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            time - time_interval.start
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
        type Presentation = DynamicTimelinePresentation<CO::ContentPresentation, ME, R>;
        // type Presentation = Self;

        fn presentation(self) -> Self::Presentation {
            DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(),
                metric: self.metric,
                rate: self.rate,
            }
        }

        // fn present(&self, time: f32, time_interval: Range<f32>) {
        //     self.content
        //         .content_present(self.rate.eval(self.metric.eval(time, time_interval)));
        // }
        // (self, time_interval: Range<f32>) -> Self::Presentation {
        //     DynamicTimelinePresentation {
        //         content_presentation: self.content.content_presentation(),
        //         rate: self.rate,
        //         start_time: time_interval.start,
        //         metric_scalar: self.metric.metric_scalar(time_interval),
        //     }
        // }
    }

    struct DynamicTimelinePresentation<CP, ME, R> {
        content_presentation: CP,
        metric: ME,
        rate: R,
        // start_time: f32,
        // metric_scalar: f32,
    }

    impl<CP, ME, R> Present for DynamicTimelinePresentation<CP, ME, R>
    where
        CP: ContentPresent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        fn present(&self, time: f32, time_interval: Range<f32>) {
            self.content_presentation
                .content_present(self.rate.eval(self.metric.eval(time, time_interval)));
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

    impl<'a, CO, ME, R> Alive<'a, DynamicTimeline<CO, ME, R>>
    where
        CO: DynamicTimelineContent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        pub fn collapse(
            self,
        ) -> Alive<'a, SteadyTimeline<<CO::ContentPresentation as Collapse>::Output>> {
            self.archive(
                |DynamicTimelinePresentation {
                     content_presentation,
                     metric,
                     rate,
                 },
                 supervisor,
                 time_interval| {
                    Alive::new(
                        SteadyTimeline {
                            mobject: content_presentation
                                .collapse(rate.eval(metric.eval(time_interval.end, time_interval))),
                        },
                        supervisor,
                    )
                },
            )
        }
    }
}

pub mod action {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::act::ApplyAct;
    use super::super::rates::Rate;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    // use super::dynamic::Collapse;
    use super::dynamic::DynamicTimeline;
    use super::dynamic::DynamicTimelineBuilder;
    use super::dynamic::DynamicTimelineContent;
    use super::dynamic::DynamicTimelineMetric;
    use super::steady::SteadyTimeline;
    // use super::steady::SteadyTimelinePresentation;
    use super::Alive;
    // use super::Present;

    pub struct ActionTimelineContent<M>
    where
        M: Mobject,
    {
        source_mobject: M,
        target_mobject: M,
        diff: M::Diff,
    }

    impl<M> DynamicTimelineContent for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = Self;

        fn content_presentation(self) -> Self::ContentPresentation {
            self
        }
    }

    impl<M> ContentPresent for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        // type ContentPresentation = ActionTimelineContentPresentation<M>;

        fn content_present(&self, time: f32) {
            self.collapse(time).render();
        }

        // fn content_present(&self, time: f32) {
        //     let mut diff = self.diff.clone();
        //     diff *= time;
        //     self.source_mobject.apply_diff(diff).render();
        // }
        // (self) -> Self::ContentPresentation {
        //     ActionTimelineContentPresentation {
        //         source_mobject: self.source_mobject,
        //         diff: self.diff,
        //     }
        // }
    }

    impl<M> Collapse for ActionTimelineContent<M>
    where
        M: Mobject,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            let mut diff = self.diff.clone();
            diff *= time;
            self.source_mobject.apply_diff(diff)
        }
    }

    // struct ActionTimelineContentPresentation<M>
    // where
    //     M: Mobject,
    // {
    //     source_mobject: M,
    //     diff: M::Diff,
    // }

    // impl<M> Present for ActionTimelineContentPresentation<M>
    // where
    //     M: Mobject,
    // {
    //     fn present(&self, time: f32) {
    //         let mut diff = self.diff.clone();
    //         diff *= time;
    //         self.source_mobject.apply_diff(diff).render();
    //     }
    // }

    // impl<M> Collapse for ActionTimelineContentPresentation<M>
    // where
    //     M: Mobject,
    // {
    //     type Output = M;

    //     fn collapse(&self, time: f32) -> Self::Output {
    //         let mut diff = self.diff.clone();
    //         diff *= time;
    //         self.source_mobject.apply_diff(diff)
    //     }
    // }

    impl<M, A, ME, R> ApplyAct<M, A> for Alive<'_, DynamicTimeline<ActionTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        A: Act<M>,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output = Self;

        fn apply_act(mut self, act: A) -> Self::Output {
            let content = &mut self.timeline.content;
            let diff = act.act(&content.target_mobject);
            content.target_mobject = content.target_mobject.apply_diff(diff.clone());
            content.diff += diff;
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
            self.steady_mobject
                .archive(|SteadyTimeline { mobject }, supervisor, _| {
                    let source_mobject = mobject.clone();
                    let diff = act.act(&source_mobject);
                    let target_mobject = source_mobject.apply_diff(diff.clone());
                    Alive::new(
                        DynamicTimeline {
                            content: ActionTimelineContent {
                                source_mobject,
                                target_mobject,
                                diff,
                            },
                            metric: self.metric,
                            rate: self.rate,
                        },
                        supervisor,
                    )
                })
        }
    }
}

pub mod continuous {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::update::Update;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;
    // use super::Present;

    pub struct ContinuousTimelineContent<M, U> {
        mobject: M,
        update: U,
    }

    impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type ContentPresentation = Self;

        fn content_presentation(self) -> Self::ContentPresentation {
            self
        }
    }

    impl<M, U> ContentPresent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        // (self) -> Self::ContentPresentation {
        //     ContinuousTimelineContentPresentation {
        //         mobject: self.mobject,
        //         update: self.update,
        //     }
        // }

        fn content_present(&self, time: f32) {
            self.collapse(time).render();
        }
    }

    // struct ContinuousTimelineContentPresentation<M, U> {
    //     mobject: M,
    //     update: U,
    // }

    // impl<M, U> Present for ContinuousTimelineContentPresentation<M, U>
    // where
    //     M: Mobject,
    //     U: Update<M>,
    // {
    //     fn present(&self, time: f32) {
    //         self.update.update(&self.mobject, time).render();
    //     }
    // }

    impl<M, U> Collapse for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            self.update.update(&self.mobject, time)
        }
    }
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::construct::Construct;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;
    use super::steady::SteadyTimeline;
    use super::Alive;
    use super::Supervisor;
    use super::SupervisorData;

    pub struct DiscreteTimelineContent<M, C> {
        mobject: M,
        construct: C,
    }

    impl<M, C> DynamicTimelineContent for DiscreteTimelineContent<M, C>
    where
        M: Mobject,
        C: Construct<M>,
    {
        type ContentPresentation = DiscreteTimelineContentPresentation<C::Output>;

        fn content_presentation(self) -> Self::ContentPresentation {
            let supervisor = Supervisor::new();
            let mobject = self
                .construct
                .construct(
                    Alive::new(
                        SteadyTimeline {
                            mobject: self.mobject,
                        },
                        &supervisor,
                    ),
                    &supervisor,
                )
                .archive(|steady_timeline, _, _| steady_timeline.mobject.clone());
            DiscreteTimelineContentPresentation {
                mobject,
                supervisor_data: supervisor.0.into_inner(),
                // construct: self.construct,
            }
        }
    }

    struct DiscreteTimelineContentPresentation<M> {
        mobject: M,
        supervisor_data: SupervisorData,
    }

    impl<M> ContentPresent for DiscreteTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        fn content_present(&self, time: f32) {
            self.supervisor_data.supervisor_present(time);
        }
    }

    impl<M> Collapse for DiscreteTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        type Output = M;

        fn collapse(&self, _time: f32) -> Self::Output {
            self.mobject.clone()
        }
    }
}
