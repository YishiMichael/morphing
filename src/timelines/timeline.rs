use super::super::toplevel::scene::Present;

pub trait Timeline: 'static {
    type Presentation: Present;

    fn presentation(self) -> Self::Presentation;
}

pub mod steady {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::toplevel::renderer::Renderer;
    use super::Present;
    use super::Timeline;

    pub struct SteadyTimeline<M> {
        pub(crate) mobject: M,
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
        fn present(&self, _time: f32, _time_interval: Range<f32>, renderer: &Renderer) {
            self.mobject.render(renderer);
        }
    }
}

pub mod dynamic {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::toplevel::renderer::Renderer;
    use super::super::rates::Rate;
    use super::Present;
    use super::Timeline;

    pub trait ContentPresent: 'static {
        fn content_present(&self, time: f32, renderer: &Renderer);
    }

    pub trait Collapse {
        type Output: Mobject;

        fn collapse(&self, time: f32) -> Self::Output;
    }

    pub trait DynamicTimelineContent: 'static {
        type ContentPresentation: ContentPresent + Collapse;

        fn content_presentation(self) -> Self::ContentPresentation;
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
        pub(crate) content: CO,
        pub(crate) metric: ME,
        pub(crate) rate: R,
    }

    impl<CO, ME, R> Timeline for DynamicTimeline<CO, ME, R>
    where
        CO: DynamicTimelineContent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Presentation = DynamicTimelinePresentation<CO::ContentPresentation, ME, R>;

        fn presentation(self) -> Self::Presentation {
            DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(),
                metric: self.metric,
                rate: self.rate,
            }
        }
    }

    pub struct DynamicTimelinePresentation<CP, ME, R> {
        pub(crate) content_presentation: CP,
        pub(crate) metric: ME,
        pub(crate) rate: R,
    }

    impl<CP, ME, R> Present for DynamicTimelinePresentation<CP, ME, R>
    where
        CP: ContentPresent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        fn present(&self, time: f32, time_interval: Range<f32>, renderer: &Renderer) {
            self.content_presentation.content_present(
                self.rate.eval(self.metric.eval(time, time_interval)),
                renderer,
            );
        }
    }
}

pub mod action {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::toplevel::renderer::Renderer;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;

    pub struct ActionTimelineContent<M>
    where
        M: Mobject,
    {
        pub(crate) source_mobject: M,
        pub(crate) target_mobject: M,
        pub(crate) diff: M::Diff,
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
        fn content_present(&self, time: f32, renderer: &Renderer) {
            self.collapse(time).render(renderer);
        }
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
}

pub mod continuous {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::toplevel::renderer::Renderer;
    use super::super::update::Update;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;

    pub struct ContinuousTimelineContent<M, U> {
        pub(crate) mobject: M,
        pub(crate) update: U,
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
        fn content_present(&self, time: f32, renderer: &Renderer) {
            self.collapse(time).render(renderer);
        }
    }

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
    use super::super::super::toplevel::renderer::Renderer;
    use super::super::super::toplevel::scene::Supervisor;
    use super::super::super::toplevel::scene::SupervisorData;
    use super::super::construct::Construct;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresent;
    use super::dynamic::DynamicTimelineContent;

    pub struct DiscreteTimelineContent<M, C> {
        pub(crate) mobject: M,
        pub(crate) construct: C,
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
                .construct(supervisor.spawn(self.mobject), &supervisor)
                .archive(|steady_timeline, _, _| steady_timeline.mobject.clone());
            DiscreteTimelineContentPresentation {
                mobject,
                supervisor_data: supervisor.into_data(),
            }
        }
    }

    pub struct DiscreteTimelineContentPresentation<M> {
        mobject: M,
        supervisor_data: SupervisorData,
    }

    impl<M> ContentPresent for DiscreteTimelineContentPresentation<M>
    where
        M: Mobject,
    {
        fn content_present(&self, time: f32, renderer: &Renderer) {
            self.supervisor_data.present_all(time, renderer);
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
