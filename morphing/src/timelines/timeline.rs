use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use crate::toplevel::world::World;

// pub trait Presentation: 'static + Send + Sync {
//     fn present(
//         &mut self,
//         time: f32,
//         time_interval: Range<f32>,
//         device: &iced::widget::shader::wgpu::Device,
//         queue: &iced::widget::shader::wgpu::Queue,
//         render_pass: &mut iced::widget::shader::wgpu::RenderPass,
//     ) -> anyhow::Result<()>;
// }

pub trait Timeline:
    'static + Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    type Presentation: Send;

    fn presentation(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Presentation;
    fn prepare(
        &self,
        time_interval: Range<f32>,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        presentation: &mut Self::Presentation,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    );
    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        presentation: &Self::Presentation,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    );
}

trait DynTimeline:
    Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    fn dyn_prepare(
        &self,
        hash: u64,
        time_interval: Range<f32>,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    );
    fn dyn_render(
        &self,
        hash: u64,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    );
}

impl<T> DynTimeline for T
where
    T: Timeline,
{
    fn dyn_prepare(
        &self,
        hash: u64,
        time_interval: Range<f32>,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        let presentation_map = match storage.get_mut::<dashmap::DashMap<u64, T::Presentation>>() {
            Some(presentation_map) => presentation_map,
            None => {
                storage.store::<dashmap::DashMap<u64, T::Presentation>>(dashmap::DashMap::new());
                storage
                    .get_mut::<dashmap::DashMap<u64, T::Presentation>>()
                    .unwrap()
            }
        };
        let mut presentation = presentation_map
            .entry(hash)
            .or_insert_with(|| self.presentation(device));
        self.prepare(
            time_interval,
            time,
            device,
            queue,
            format,
            &mut presentation,
            bounds,
            viewport,
        );
    }

    fn dyn_render(
        &self,
        hash: u64,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let presentation_map = storage
            .get::<dashmap::DashMap<u64, T::Presentation>>()
            .unwrap();
        let presentation = presentation_map.get(&hash).unwrap();
        self.render(encoder, &presentation, target, clip_bounds);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry {
    hash: u64,
    time_interval: Range<f32>,
    #[serde(with = "serde_traitobject")]
    timeline: Box<dyn DynTimeline>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct TimelineEntries(Arc<Vec<TimelineEntry>>);

impl TimelineEntries {
    // pub(crate) fn new(entries: Vec<TimelineEntry>) -> Self {
    //     Self(Arc::new(entries))
    // }

    // pub(crate) fn push<T>(&mut self, time_interval: Range<f32>, timeline: T)
    // where
    //     T: 'static + Timeline,
    // {
    //     // Hash `Box<T>` instead of `T`.
    //     // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
    //     let timeline =
    //         serde_traitobject::Box::new(timeline) as serde_traitobject::Box<dyn DynTimeline>;
    //     let hash = seahash::hash(&ron::ser::to_string(&timeline).unwrap().into_bytes());
    //     self.0.push(TimelineEntry {
    //         hash,
    //         time_interval,
    //         timeline: timeline.into_box(),
    //     });
    // }

    fn prepare(
        &self,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        for timeline_entry in &*self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry.timeline.dyn_prepare(
                    timeline_entry.hash,
                    timeline_entry.time_interval.clone(),
                    time,
                    device,
                    queue,
                    format,
                    storage,
                    bounds,
                    viewport,
                );
            }
        }
    }

    fn render(
        &self,
        time: f32,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        for timeline_entry in &*self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry.timeline.dyn_render(
                    timeline_entry.hash,
                    encoder,
                    storage,
                    target,
                    clip_bounds,
                );
            }
        }
    }
}

pub struct Supervisor<'w> {
    world: &'w World,
    time: RefCell<Arc<f32>>,
    timeline_entries: RefCell<Vec<TimelineEntry>>,
}

impl<'w> Supervisor<'w> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            time: RefCell::new(Arc::new(0.0)),
            timeline_entries: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn world(&self) -> &World {
        &self.world
    }

    pub(crate) fn time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub(crate) fn push<T>(&self, time_interval: Range<f32>, timeline: T)
    where
        T: 'static + Timeline,
    {
        // Hash `Box<T>` instead of `T`.
        // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
        let timeline =
            serde_traitobject::Box::new(timeline) as serde_traitobject::Box<dyn DynTimeline>;
        let hash = seahash::hash(&ron::ser::to_string(&timeline).unwrap().into_bytes());
        self.timeline_entries.borrow_mut().push(TimelineEntry {
            hash,
            time_interval,
            timeline: timeline.into_box(),
        });
    }

    pub(crate) fn into_timeline_entries(self) -> TimelineEntries {
        TimelineEntries(Arc::new(self.timeline_entries.into_inner()))
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative argument `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

// TODO
#[derive(Debug)]
struct TimelineEntriesStamp {
    timeline_entries: TimelineEntries,
    time: f32,
}

impl iced::widget::shader::Primitive for TimelineEntriesStamp {
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        self.timeline_entries
            .prepare(self.time, device, queue, format, storage, bounds, viewport);
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        self.timeline_entries
            .render(self.time, encoder, storage, target, clip_bounds);
    }
}

// struct PresentationEntry {
//     time_interval: Range<f32>,
//     presentation: Box<dyn Presentation>,
// }

// pub(crate) struct PresentationEntries(Vec<PresentationEntry>);

// impl PresentationEntries {
//     pub(crate) fn present(
//         &mut self,
//         time: f32,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         render_pass: &mut wgpu::RenderPass,
//     ) -> anyhow::Result<()> {
//         for presentation_entry in &mut self.0 {
//             if presentation_entry.time_interval.contains(&time) {
//                 presentation_entry.presentation.present(
//                     time,
//                     presentation_entry.time_interval.clone(),
//                     device,
//                     queue,
//                     render_pass,
//                 )?;
//             }
//         }
//         Ok(())
//     }
// }

pub mod steady {
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectPresentation;
    use super::Timeline;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct SteadyTimeline<M> {
        pub(crate) mobject: M,
    }

    impl<M> Timeline for SteadyTimeline<M>
    where
        M: Mobject,
    {
        type Presentation = M::MobjectPresentation;

        fn presentation(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Presentation {
            self.mobject.presentation(device)
        }

        fn prepare(
            &self,
            _time_interval: Range<f32>,
            _time: f32,
            _device: &iced::widget::shader::wgpu::Device,
            _queue: &iced::widget::shader::wgpu::Queue,
            _format: iced::widget::shader::wgpu::TextureFormat,
            _presentation: &mut Self::Presentation,
            _bounds: &iced::Rectangle,
            _viewport: &iced::widget::shader::Viewport,
        ) {
        }

        fn render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::Presentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            presentation.render(encoder, target, clip_bounds);
        }
    }
}

pub mod dynamic {
    use std::fmt::Debug;
    use std::ops::Range;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectPresentation;
    use super::super::rates::Rate;
    // use super::Presentation;
    use super::Timeline;

    // pub trait ContentPresentation: 'static + Send + Sync {
    //     fn content_present(
    //         &mut self,
    //         time: f32,
    //         device: &iced::widget::shader::wgpu::Device,
    //         queue: &iced::widget::shader::wgpu::Queue,
    //         render_pass: &mut iced::widget::shader::wgpu::RenderPass,
    //     ) -> anyhow::Result<()>;
    // }

    pub trait DynamicTimelineContent:
        'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
    {
        type ContentPresentation: Send;
        type CollapseOutput: Mobject;

        fn content_presentation(
            &self,
            device: &iced::widget::shader::wgpu::Device,
        ) -> Self::ContentPresentation;
        fn content_prepare(
            &self,
            time: f32,
            device: &iced::widget::shader::wgpu::Device,
            queue: &iced::widget::shader::wgpu::Queue,
            format: iced::widget::shader::wgpu::TextureFormat,
            presentation: &mut Self::ContentPresentation,
            bounds: &iced::Rectangle,
            viewport: &iced::widget::shader::Viewport,
        );
        fn content_render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::ContentPresentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        );
        fn content_collapse(self, time: f32) -> Self::CollapseOutput;
    }

    pub trait DynamicTimelineMetric:
        'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
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

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
        //fn prepare(&self, device: &wgpu::Device) -> anyhow::Result<Box<dyn Presentation>> {
        //    Ok(Box::new(DynamicTimelinePresentation {
        //        content_presentation: self.content.content_prepare(device)?,
        //        metric: self.metric.clone(),
        //        rate: self.rate.clone(),
        //    }))
        //}

        type Presentation = CO::ContentPresentation;

        fn presentation(&self, device: &iced::widget::shader::wgpu::Device) -> Self::Presentation {
            self.content.content_presentation(device)
        }

        fn prepare(
            &self,
            time_interval: Range<f32>,
            time: f32,
            device: &iced::widget::shader::wgpu::Device,
            queue: &iced::widget::shader::wgpu::Queue,
            format: iced::widget::shader::wgpu::TextureFormat,
            presentation: &mut Self::Presentation,
            bounds: &iced::Rectangle,
            viewport: &iced::widget::shader::Viewport,
        ) {
            self.content.content_prepare(
                self.rate.eval(self.metric.eval(time, time_interval)),
                device,
                queue,
                format,
                presentation,
                bounds,
                viewport,
            );
        }

        fn render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::Presentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            // let mut render_pass =
            //     encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
            //         label: None,
            //         color_attachments: &[Some(
            //             iced::widget::shader::wgpu::RenderPassColorAttachment {
            //                 view: target,
            //                 resolve_target: None,
            //                 ops: iced::widget::shader::wgpu::Operations {
            //                     load: iced::widget::shader::wgpu::LoadOp::Load,
            //                     store: iced::widget::shader::wgpu::StoreOp::Store,
            //                 },
            //             },
            //         )],
            //         depth_stencil_attachment: None,
            //         timestamp_writes: None,
            //         occlusion_query_set: None,
            //     });
            // render_pass.set_scissor_rect(
            //     clip_bounds.x,
            //     clip_bounds.y,
            //     clip_bounds.width,
            //     clip_bounds.height,
            // );
            self.content
                .content_render(encoder, presentation, target, clip_bounds);
        }
    }

    // impl<CP, ME, R> Presentation for DynamicTimelinePresentation<CP, ME, R>
    // where
    //     CP: ContentPresentation,
    //     ME: DynamicTimelineMetric,
    //     R: Rate,
    // {
    //     fn present(
    //         &mut self,
    //         time: f32,
    //         time_interval: Range<f32>,
    //         device: &wgpu::Device,
    //         queue: &wgpu::Queue,
    //         render_pass: &mut wgpu::RenderPass,
    //     ) -> anyhow::Result<()> {
    //         self.content_presentation.content_present(
    //             self.rate.eval(self.metric.eval(time, time_interval)),
    //             device,
    //             queue,
    //             render_pass,
    //         )?;
    //         Ok(())
    //     }
    // }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct IndeterminedTimelineContent<M> {
        pub(crate) mobject: M,
    }

    impl<M> DynamicTimelineContent for IndeterminedTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = M::MobjectPresentation;
        type CollapseOutput = M;

        fn content_presentation(
            &self,
            device: &iced::widget::shader::wgpu::Device,
        ) -> Self::ContentPresentation {
            self.mobject.presentation(device)
        }

        fn content_prepare(
            &self,
            _time: f32,
            _device: &iced::widget::shader::wgpu::Device,
            _queue: &iced::widget::shader::wgpu::Queue,
            _format: iced::widget::shader::wgpu::TextureFormat,
            _presentation: &mut Self::ContentPresentation,
            _bounds: &iced::Rectangle,
            _viewport: &iced::widget::shader::Viewport,
        ) {
        }

        fn content_render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::ContentPresentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            presentation.render(encoder, target, clip_bounds);
        }

        fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
            self.mobject
        }
    }

    // pub struct IndeterminedTimelineContentPresentation<MR> {
    //     realization: MR,
    // }

    // impl<MR> ContentPresentation for IndeterminedTimelineContentPresentation<MR>
    // where
    //     MR: MobjectRealization,
    // {
    //     fn content_present(
    //         &mut self,
    //         _time: f32,
    //         _device: &wgpu::Device,
    //         _queue: &wgpu::Queue,
    //         render_pass: &mut wgpu::RenderPass,
    //     ) -> anyhow::Result<()> {
    //         self.realization.render(render_pass)?;
    //         Ok(())
    //     }
    // }
}

pub mod action {
    use crate::mobjects::mobject::MobjectPresentation;

    use super::super::super::mobjects::mobject::Mobject;
    // use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::act::MobjectDiff;
    // use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct ActionTimelineContent<M, D> {
        pub(crate) mobject: M,
        pub(crate) diff: D,
    }

    impl<M, MD> DynamicTimelineContent for ActionTimelineContent<M, MD>
    where
        M: Mobject,
        MD: MobjectDiff<M>,
    {
        type ContentPresentation = M::MobjectPresentation;
        type CollapseOutput = M;

        fn content_presentation(
            &self,
            device: &iced::widget::shader::wgpu::Device,
        ) -> Self::ContentPresentation {
            self.mobject.presentation(device)
        }

        fn content_prepare(
            &self,
            time: f32,
            _device: &iced::widget::shader::wgpu::Device,
            queue: &iced::widget::shader::wgpu::Queue,
            _format: iced::widget::shader::wgpu::TextureFormat,
            presentation: &mut Self::ContentPresentation,
            _bounds: &iced::Rectangle,
            _viewport: &iced::widget::shader::Viewport,
        ) {
            self.diff
                .apply_presentation(presentation, &self.mobject, time, queue);
        }

        fn content_render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::ContentPresentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            presentation.render(encoder, target, clip_bounds);
        }

        fn content_collapse(self, time: f32) -> Self::CollapseOutput {
            let mut mobject = self.mobject;
            self.diff.apply(&mut mobject, time);
            mobject
        }

        // fn content_prepare(
        //     &self,
        //     device: &wgpu::Device,
        // ) -> anyhow::Result<Self::ContentPresentation> {
        //     Ok(ActionTimelineContentPresentation {
        //         realization: self.mobject.realize(device)?,
        //         reference_mobject: self.mobject.clone(),
        //         diff: self.diff.clone(),
        //     })
        // }
    }

    // pub struct ActionTimelineContentPresentation<MR, M, MD> {
    //     realization: MR,
    //     reference_mobject: M,
    //     diff: MD,
    // }

    // impl<M, MD> ContentPresentation for ActionTimelineContentPresentation<M::MobjectPresentation, M, MD>
    // where
    //     M: Mobject,
    //     MD: MobjectDiff<M>,
    // {
    //     fn content_present(
    //         &mut self,
    //         time: f32,
    //         _device: &wgpu::Device,
    //         queue: &wgpu::Queue,
    //         render_pass: &mut wgpu::RenderPass,
    //     ) -> anyhow::Result<()> {
    //         self.diff.apply_presentation(
    //             &mut self.realization,
    //             &self.reference_mobject,
    //             time,
    //             queue,
    //         )?;
    //         self.realization.render(render_pass)?;
    //         Ok(())
    //     }
    // }
}

pub mod continuous {
    use crate::mobjects::mobject::MobjectPresentation;

    use super::super::super::mobjects::mobject::Mobject;
    // use super::super::super::mobjects::mobject::MobjectRealization;
    use super::super::update::Update;
    // use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;
    // use super::dynamic::TimelineContentCollapse;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct ContinuousTimelineContent<M, U> {
        pub(crate) mobject: M,
        pub(crate) update: U,
    }

    impl<M, U> DynamicTimelineContent for ContinuousTimelineContent<M, U>
    where
        M: Mobject,
        U: Update<M>,
    {
        type ContentPresentation = M::MobjectPresentation;
        type CollapseOutput = M;

        fn content_presentation(
            &self,
            device: &iced::widget::shader::wgpu::Device,
        ) -> Self::ContentPresentation {
            self.mobject.presentation(device)
        }

        fn content_prepare(
            &self,
            time: f32,
            device: &iced::widget::shader::wgpu::Device,
            queue: &iced::widget::shader::wgpu::Queue,
            _format: iced::widget::shader::wgpu::TextureFormat,
            presentation: &mut Self::ContentPresentation,
            _bounds: &iced::Rectangle,
            _viewport: &iced::widget::shader::Viewport,
        ) {
            self.update
                .update_presentation(presentation, &self.mobject, time, device, queue);
        }

        fn content_render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::ContentPresentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            presentation.render(encoder, target, clip_bounds);
        }

        // fn content_prepare(
        //     &self,
        //     device: &wgpu::Device,
        // ) -> anyhow::Result<Self::ContentPresentation> {
        //     Ok(ContinuousTimelineContentPresentation {
        //         realization: self.mobject.realize(device)?,
        //         reference_mobject: self.mobject.clone(),
        //         update: self.update.clone(),
        //     })
        // }

        fn content_collapse(self, time: f32) -> Self::CollapseOutput {
            let mut mobject = self.mobject;
            self.update.update(&mut mobject, time);
            mobject
        }
    }

    // pub struct ContinuousTimelineContentPresentation<MR, M, U> {
    //     realization: MR,
    //     reference_mobject: M,
    //     update: U,
    // }

    // impl<MR, M, U> ContentPresentation for ContinuousTimelineContentPresentation<MR, M, U>
    // where
    //     MR: MobjectRealization,
    //     M: Mobject<Realization = MR>,
    //     U: Update<M>,
    // {
    //     fn content_present(
    //         &mut self,
    //         time: f32,
    //         device: &wgpu::Device,
    //         queue: &wgpu::Queue,
    //         render_pass: &mut wgpu::RenderPass,
    //     ) -> anyhow::Result<()> {
    //         self.update.update_presentation(
    //             &mut self.realization,
    //             &self.reference_mobject,
    //             time,
    //             device,
    //             queue,
    //         )?;
    //         self.realization.render(render_pass)?;
    //         Ok(())
    //     }
    // }
}

pub mod discrete {
    use super::super::super::mobjects::mobject::Mobject;
    // use super::dynamic::ContentPresentation;
    use super::dynamic::DynamicTimelineContent;
    // use super::dynamic::TimelineContentCollapse;
    // use super::PresentationEntries;
    use super::TimelineEntries;

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct DiscreteTimelineContent<M> {
        pub(crate) mobject: M,
        pub(crate) timeline_entries: TimelineEntries,
    }

    impl<M> DynamicTimelineContent for DiscreteTimelineContent<M>
    where
        M: Mobject,
    {
        type ContentPresentation = (iced::widget::shader::Storage, f32); // ???
        type CollapseOutput = M;

        fn content_presentation(
            &self,
            _device: &iced::widget::shader::wgpu::Device,
        ) -> Self::ContentPresentation {
            (iced::widget::shader::Storage::default(), 0.0)
        }

        fn content_prepare(
            &self,
            time: f32,
            device: &iced::widget::shader::wgpu::Device,
            queue: &iced::widget::shader::wgpu::Queue,
            format: iced::widget::shader::wgpu::TextureFormat,
            presentation: &mut Self::ContentPresentation,
            bounds: &iced::Rectangle,
            viewport: &iced::widget::shader::Viewport,
        ) {
            self.timeline_entries.prepare(
                time,
                device,
                queue,
                format,
                &mut presentation.0,
                bounds,
                viewport,
            );
            *&mut presentation.1 = time;
        }

        fn content_render(
            &self,
            encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
            presentation: &Self::ContentPresentation,
            target: &iced::widget::shader::wgpu::TextureView,
            clip_bounds: &iced::Rectangle<u32>,
        ) {
            self.timeline_entries.render(
                presentation.1,
                encoder,
                &presentation.0,
                target,
                clip_bounds,
            );
        }

        // fn content_prepare(
        //     &self,
        //     device: &wgpu::Device,
        // ) -> anyhow::Result<Self::ContentPresentation> {
        //     Ok(DiscreteTimelineContentPresentation {
        //         presentation_entries: self.timeline_entries.prepare(device)?,
        //     })
        // }

        fn content_collapse(self, _time: f32) -> Self::CollapseOutput {
            self.mobject
        }
    }

    // pub struct DiscreteTimelineContentPresentation {
    //     presentation_entries: PresentationEntries,
    // }

    // impl ContentPresentation for DiscreteTimelineContentPresentation {
    //     fn content_present(
    //         &mut self,
    //         time: f32,
    //         device: &wgpu::Device,
    //         queue: &wgpu::Queue,
    //         render_pass: &mut wgpu::RenderPass,
    //     ) -> anyhow::Result<()> {
    //         self.presentation_entries
    //             .present(time, device, queue, render_pass)?;
    //         Ok(())
    //     }
    // }
}
