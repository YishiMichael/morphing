use std::fmt::Debug;
// use std::sync::LazyLock;

use super::super::toplevel::scene::Presentation;

pub trait Timeline:
    'static + Debug + serde_traitobject::Serialize + serde_traitobject::Deserialize
{
    // fn id(&self) -> &'static str;
    fn presentation<'t>(&'t self, device: &wgpu::Device) -> Box<dyn 't + Presentation>;
}

// static TIMELINE_REGISTRY: LazyLock<serde_flexitos::MapRegistry<dyn Timeline>> =
//     LazyLock::new(|| {
//         let mut registry = serde_flexitos::MapRegistry::<dyn Timeline>::new("timeline");
//         // TODO: the hard part, register a deserializer that can handle Constant<T> generically.
//         //registry.register(Constant::ID, |d| Ok(Box::new(erased_serde::deserialize::<Constant<T>>(d)?)));
//         registry
//     });

pub mod steady {
    use std::ops::Range;

    use serde::Deserialize;
    use serde::Serialize;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::Presentation;
    use super::Timeline;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SteadyTimeline<M> {
        pub(crate) mobject: M,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        // type Presentation<'t> = SteadyTimelinePresentation<M::Realization> where M: 't;

        fn presentation<'t>(&'t self, device: &wgpu::Device) -> Box<dyn 't + Presentation> {
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

    use serde::Deserialize;
    use serde::Serialize;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::rates::Rate;
    use super::Presentation;
    use super::Timeline;

    pub trait ContentPresentation {
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
        'static + Debug + serde::de::DeserializeOwned + Serialize + Collapse
    {
        type ContentPresentation<'t>: 't + ContentPresentation
        where
            Self: 't;

        fn content_presentation<'t>(
            &'t self,
            device: &wgpu::Device,
        ) -> Self::ContentPresentation<'t>;
    }

    pub trait DynamicTimelineMetric:
        'static + Debug + serde::de::DeserializeOwned + Serialize
    {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32;
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct RelativeTimelineMetric;

    impl DynamicTimelineMetric for RelativeTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            (time - time_interval.start) / (time_interval.end - time_interval.start)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct AbsoluteTimelineMetric;

    impl DynamicTimelineMetric for AbsoluteTimelineMetric {
        fn eval(&self, time: f32, time_interval: Range<f32>) -> f32 {
            time - time_interval.start
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
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
        // type Presentation<'t> = DynamicTimelinePresentation<'t, CO::ContentPresentation<'t>, ME, R> where CO: 't, ME: 't, R: 't;

        fn presentation<'t>(&'t self, device: &wgpu::Device) -> Box<dyn 't + Presentation> {
            Box::new(DynamicTimelinePresentation {
                content_presentation: self.content.content_presentation(device),
                metric: &self.metric,
                rate: &self.rate,
            })
        }
    }

    pub struct DynamicTimelinePresentation<'t, CP, ME, R> {
        pub(crate) content_presentation: CP,
        pub(crate) metric: &'t ME,
        pub(crate) rate: &'t R,
    }

    impl<CP, ME, R> Presentation for DynamicTimelinePresentation<'_, CP, ME, R>
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

    use serde::Deserialize;
    use serde::Serialize;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::act::MobjectDiff;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ActionTimelineContent<M, D> {
        pub(crate) mobject: M,
        // pub(crate) source_mobject: M,
        // pub(crate) target_mobject: M,
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
        type ContentPresentation<'t> = ActionTimelineContentPresentation<'t, M::Realization, M, MD> where M: 't, MD: 't;

        fn content_presentation<'t>(
            &'t self,
            device: &wgpu::Device,
        ) -> Self::ContentPresentation<'t> {
            ActionTimelineContentPresentation {
                realization: RefCell::new(self.mobject.realize(device)),
                reference_mobject: &self.mobject,
                diff: &self.diff,
            }
        }
    }

    pub struct ActionTimelineContentPresentation<'t, MR, M, MD> {
        realization: RefCell<MR>,
        reference_mobject: &'t M,
        diff: &'t MD,
    }

    impl<M, MD> ContentPresentation for ActionTimelineContentPresentation<'_, M::Realization, M, MD>
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
            // self.collapse(time).render(renderer);
        }
    }
}

pub mod continuous {
    use std::cell::RefCell;

    use serde::Deserialize;
    use serde::Serialize;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::update::Update;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Debug, Deserialize, Serialize)]
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
        type ContentPresentation<'t> =
            ContinuousTimelineContentPresentation<'t, M::Realization, M, U> where M: 't, U: 't;

        fn content_presentation<'t>(
            &'t self,
            device: &wgpu::Device,
        ) -> Self::ContentPresentation<'t> {
            ContinuousTimelineContentPresentation {
                realization: RefCell::new(self.mobject.realize(device)),
                reference_mobject: &self.mobject,
                update: &self.update,
            }
        }
    }

    pub struct ContinuousTimelineContentPresentation<'t, MR, M, U> {
        realization: RefCell<MR>,
        reference_mobject: &'t M,
        update: &'t U,
    }

    impl<M, U> ContentPresentation for ContinuousTimelineContentPresentation<'_, M::Realization, M, U>
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
    use serde::Deserialize;
    use serde::Serialize;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::toplevel::scene::PresentationCollection;
    use super::super::super::toplevel::scene::TimelineCollection;
    use super::dynamic::Collapse;
    use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DiscreteTimelineContent<M> {
        pub(crate) mobject: M,
        pub(crate) timeline_collection: TimelineCollection,
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
        type ContentPresentation<'t> = DiscreteTimelineContentPresentation<'t> where M: 't;

        fn content_presentation<'t>(
            &'t self,
            device: &wgpu::Device,
        ) -> Self::ContentPresentation<'t> {
            DiscreteTimelineContentPresentation {
                presentation_collection: self.timeline_collection.presentation_collection(device),
            }
        }
    }

    pub struct DiscreteTimelineContentPresentation<'t> {
        presentation_collection: PresentationCollection<'t>,
    }

    impl ContentPresentation for DiscreteTimelineContentPresentation<'_> {
        fn content_present(
            &self,
            time: f32,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            render_pass: &mut wgpu::RenderPass,
        ) {
            self.presentation_collection
                .present_collection(time, device, queue, render_pass);
        }
    }
}
