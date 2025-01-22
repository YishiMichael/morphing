use std::fmt::Debug;
use std::ops::Range;

pub trait Timeline:
    'static + Debug + serde_traitobject::Serialize + serde_traitobject::Deserialize
{
    fn presentation(&self, device: &wgpu::Device) -> Box<dyn Presentation>;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry {
    time_interval: Range<f32>,
    #[serde(with = "serde_traitobject")]
    timeline: Box<dyn Timeline>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct TimelineEntries(Vec<TimelineEntry>);

impl TimelineEntries {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn push<T>(&mut self, time_interval: Range<f32>, timeline: T)
    where
        T: Timeline,
    {
        self.0.push(TimelineEntry {
            time_interval,
            timeline: Box::new(timeline),
        });
    }

    pub(crate) fn presentation(&self, device: &wgpu::Device) -> PresentationEntries {
        PresentationEntries(
            self.0
                .iter()
                .map(|timeline_entry| PresentationEntry {
                    time_interval: timeline_entry.time_interval.clone(),
                    presentation: timeline_entry.timeline.presentation(device),
                })
                .collect(),
        )
    }
}

pub trait Presentation: 'static {
    fn present(
        &self,
        time: f32,
        time_interval: Range<f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
    );
}

struct PresentationEntry {
    time_interval: Range<f32>,
    presentation: Box<dyn Presentation>,
}

pub(crate) struct PresentationEntries(Vec<PresentationEntry>);

impl PresentationEntries {
    pub(crate) fn present(
        &self,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
    ) {
        for presentation_entry in &self.0 {
            if presentation_entry.time_interval.contains(&time) {
                presentation_entry.presentation.present(
                    time,
                    presentation_entry.time_interval.clone(),
                    device,
                    queue,
                    render_pass,
                );
            }
        }
    }
}

pub mod steady {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::Presentation;
    use super::Timeline;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct SteadyTimeline<M> {
        pub(crate) mobject: M,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        fn presentation(&self, device: &wgpu::Device) -> Box<dyn Presentation> {
            Box::new(SteadyTimelinePresentation {
                realization: self.mobject.realize(device),
            })
        }
    }

    pub struct SteadyTimelinePresentation<MR> {
        realization: MR,
    }

    impl<MR> Presentation for SteadyTimelinePresentation<MR>
    where
        MR: MobjectRealization,
    {
        fn present(
            &self,
            _time: f32,
            _time_interval: Range<f32>,
            _device: &wgpu::Device,
            _queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            self.realization.render(render_pass);
        }
    }
}

pub mod dynamic {
    use std::fmt::Debug;
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::Rate;
    use super::Presentation;
    use super::Timeline;

    pub trait ContentPresentation: 'static {
        fn content_present(
            &self,
            time: f32,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        );
    }

    pub trait Collapse {
        type Output: Mobject;

        fn collapse(&self, time: f32) -> Self::Output;
    }

    pub trait DynamicTimelineContent:
        'static + Debug + serde::de::DeserializeOwned + serde::Serialize + Collapse
    {
        type ContentPresentation: ContentPresentation;

        fn content_presentation(&self, device: &wgpu::Device) -> Self::ContentPresentation;
    }

    pub trait DynamicTimelineMetric:
        'static + Clone + Debug + serde::de::DeserializeOwned + serde::Serialize
    {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32;
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            (time - time_interval.start) / (time_interval.end - time_interval.start)
        }
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            time - time_interval.start
        }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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
        fn presentation(&self, device: &wgpu::Device) -> Box<dyn Presentation> {
            Box::new(DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(device),
                metric: self.metric.clone(),
                rate: self.rate.clone(),
            })
        }
    }

    pub struct DynamicTimelinePresentation<CP, ME, R> {
        content_presentation: CP,
        metric: ME,
        rate: R,
    }

    impl<CP, ME, R> Presentation for DynamicTimelinePresentation<CP, ME, R>
    where
        CP: ContentPresentation,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        fn present(
            &self,
            time: f32,
            time_interval: Range<f32>,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            self.content_presentation.content_present(
                self.rate.eval(self.metric.eval(time, time_interval)),
                device,
                queue,
                render_pass,
            );
        }
    }
}

pub mod action {
    use std::cell::RefCell;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::act::MobjectDiff;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct ActionTimelineContent<M, D> {
        pub(crate) mobject: M,
        pub(crate) diff: D,
    }

    impl<M, MD> Collapse for ActionTimelineContent<M, MD>
    where
        M: Mobject,
        MD: MobjectDiff<M>,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            let mut mobject = self.mobject.clone();
            self.diff.apply(&mut mobject, time);
            mobject
        }
    }

    impl<M, MD> DynamicTimelineContent for ActionTimelineContent<M, MD>
    where
        M: Mobject,
        MD: MobjectDiff<M>,
    {
        type ContentPresentation = ActionTimelineContentPresentation<M::Realization, M, MD>;

        fn content_presentation(&self, device: &wgpu::Device) -> Self::ContentPresentation {
            ActionTimelineContentPresentation {
                realization: RefCell::new(self.mobject.realize(device)),
                reference_mobject: self.mobject.clone(),
                diff: self.diff.clone(),
            }
        }
    }

    pub struct ActionTimelineContentPresentation<MR, M, MD> {
        realization: RefCell<MR>,
        reference_mobject: M,
        diff: MD,
    }

    impl<M, MD> ContentPresentation for ActionTimelineContentPresentation<M::Realization, M, MD>
    where
        M: Mobject,
        MD: MobjectDiff<M>,
    {
        fn content_present(
            &self,
            time: f32,
            _device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            let mut realization = self.realization.borrow_mut();
            self.diff
                .apply_realization(&mut realization, &self.reference_mobject, time, queue);
            realization.render(render_pass);
        }
    }
}

pub mod continuous {
    use std::cell::RefCell;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::update::Update;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct ContinuousTimelineContent<M, U> {
        pub(crate) mobject: M,
        pub(crate) update: U,
    }

    impl<M, U> Collapse for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type Output = M;

        fn collapse(&self, time: f32) -> Self::Output {
            let mut mobject = self.mobject.clone();
            self.update.update(&mut mobject, time);
            mobject
        }
    }

    impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type ContentPresentation = ContinuousTimelineContentPresentation<M::Realization, M, U>;

        fn content_presentation(&self, device: &wgpu::Device) -> Self::ContentPresentation {
            ContinuousTimelineContentPresentation {
                realization: RefCell::new(self.mobject.realize(device)),
                reference_mobject: self.mobject.clone(),
                update: self.update.clone(),
            }
        }
    }

    pub struct ContinuousTimelineContentPresentation<MR, M, U> {
        realization: RefCell<MR>,
        reference_mobject: M,
        update: U,
    }

    impl<M, U> ContentPresentation for ContinuousTimelineContentPresentation<M::Realization, M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        fn content_present(
            &self,
            time: f32,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            let mut realization = self.realization.borrow_mut();
            self.update.update_realization(
                &mut realization,
                &self.reference_mobject,
                time,
                device,
                queue,
            );
            realization.render(render_pass);
        }
    }
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;
    use super::PresentationEntries;
    use super::TimelineEntries;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct DiscreteTimelineContent<M> {
        pub(crate) mobject: M,
        pub(crate) timeline_entries: TimelineEntries,
    }

    impl<M> Collapse for DiscreteTimelineContent<M>
    where
        M: Mobject,
    {
        type Output = M;

        fn collapse(&self, _time: f32) -> Self::Output {
            self.mobject.clone()
        }
    }

    impl<M> DynamicTimelineContent for DiscreteTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = DiscreteTimelineContentPresentation;

        fn content_presentation(&self, device: &wgpu::Device) -> Self::ContentPresentation {
            DiscreteTimelineContentPresentation {
                presentation_entries: self.timeline_entries.presentation(device),
            }
        }
    }

    pub struct DiscreteTimelineContentPresentation {
        presentation_entries: PresentationEntries,
    }

    impl ContentPresentation for DiscreteTimelineContentPresentation {
        fn content_present(
            &self,
            time: f32,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            self.presentation_entries
                .present(time, device, queue, render_pass);
        }
    }
}
