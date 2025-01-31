use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

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
        presentation: &mut Self::Presentation,
    );
    fn render(
        &self,
        render_pass: &mut iced::widget::shader::wgpu::RenderPass,
        presentation: &Self::Presentation,
    );
}

trait DynTimeline:
    Send + Sync + Debug + serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    fn dyn_prepare(
        self: Arc<Self>,
        time_interval: Range<f32>,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        storage: &mut iced::widget::shader::Storage,
    );
    fn dyn_render(
        self: Arc<Self>,
        render_pass: &mut iced::widget::shader::wgpu::RenderPass,
        storage: &iced::widget::shader::Storage,
    );
}

impl<T> DynTimeline for T
where
    T: Timeline,
{
    fn dyn_prepare(
        self: Arc<Self>,
        time_interval: Range<f32>,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        storage: &mut iced::widget::shader::Storage,
    ) {
        let presentation_map = match storage.get_mut::<dashmap::DashMap<Arc<T>, T::Presentation>>()
        {
            Some(presentation_map) => presentation_map,
            None => {
                storage.store::<dashmap::DashMap<Arc<T>, T::Presentation>>(dashmap::DashMap::new());
                storage
                    .get_mut::<dashmap::DashMap<Arc<T>, T::Presentation>>()
                    .unwrap()
            }
        };
        let mut presentation = presentation_map
            .entry(self.clone())
            .or_insert_with(|| self.presentation(device));
        self.prepare(time_interval, time, device, queue, &mut presentation);
    }

    fn dyn_render(
        self: Arc<Self>,
        render_pass: &mut iced::widget::shader::wgpu::RenderPass,
        storage: &iced::widget::shader::Storage,
    ) {
        let presentation_map = storage
            .get::<dashmap::DashMap<Arc<T>, T::Presentation>>()
            .unwrap();
        let presentation = presentation_map.get(&self).unwrap();
        self.render(render_pass, &presentation);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TimelineEntry {
    time_interval: Range<f32>,
    #[serde(with = "serde_traitobject")]
    timeline: Arc<dyn DynTimeline>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct TimelineEntries(Vec<TimelineEntry>);

impl TimelineEntries {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn push<T>(&mut self, time_interval: Range<f32>, timeline: T)
    where
        T: 'static + Timeline,
    {
        self.0.push(TimelineEntry {
            time_interval,
            timeline: Arc::new(timeline),
        });
    }

    fn prepare(
        &self,
        time: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        storage: &mut iced::widget::shader::Storage,
    ) {
        for timeline_entry in &self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry.timeline.clone().dyn_prepare(
                    timeline_entry.time_interval.clone(),
                    time,
                    device,
                    queue,
                    storage,
                );
            }
        }
    }

    fn render(
        &self,
        time: f32,
        render_pass: &mut iced::widget::shader::wgpu::RenderPass,
        storage: &iced::widget::shader::Storage,
    ) {
        for timeline_entry in &self.0 {
            if timeline_entry.time_interval.contains(&time) {
                timeline_entry
                    .timeline
                    .clone()
                    .dyn_render(render_pass, storage);
            }
        }
    }
}

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
        _format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        self.timeline_entries
            .prepare(self.time, device, queue, storage);
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    iced::widget::shader::wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: iced::widget::shader::wgpu::Operations {
                            load: iced::widget::shader::wgpu::LoadOp::Load,
                            store: iced::widget::shader::wgpu::StoreOp::Store,
                        },
                    },
                )],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );
        self.timeline_entries
            .render(self.time, &mut render_pass, storage);
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
            _presentation: &mut Self::Presentation,
        ) {
        }

        fn render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::Presentation,
        ) {
            presentation.render(render_pass);
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
        'static
        //+ Clone
        + Send
        + Sync
        + Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
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
            presentation: &mut Self::ContentPresentation,
        );
        fn content_render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::ContentPresentation,
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
            presentation: &mut Self::Presentation,
        ) {
            self.content.content_prepare(
                self.rate.eval(self.metric.eval(time, time_interval)),
                device,
                queue,
                presentation,
            );
        }

        fn render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::Presentation,
        ) {
            self.content.content_render(render_pass, presentation);
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
            _presentation: &mut Self::ContentPresentation,
        ) {
        }

        fn content_render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::ContentPresentation,
        ) {
            presentation.render(render_pass);
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

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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
            presentation: &mut Self::ContentPresentation,
        ) {
            self.diff
                .apply_presentation(presentation, &self.mobject, time, queue);
        }

        fn content_render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::ContentPresentation,
        ) {
            presentation.render(render_pass);
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

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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
            presentation: &mut Self::ContentPresentation,
        ) {
            self.update
                .update_presentation(presentation, &self.mobject, time, device, queue);
        }

        fn content_render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::ContentPresentation,
        ) {
            presentation.render(render_pass);
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

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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
            presentation: &mut Self::ContentPresentation,
        ) {
            self.timeline_entries
                .prepare(time, device, queue, &mut presentation.0);
            *&mut presentation.1 = time;
        }

        fn content_render(
            &self,
            render_pass: &mut iced::widget::shader::wgpu::RenderPass,
            presentation: &Self::ContentPresentation,
        ) {
            self.timeline_entries
                .render(presentation.1, render_pass, &presentation.0);
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
