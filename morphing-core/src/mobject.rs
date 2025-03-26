use core::range::Range;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use crate::timer::RateChain;
use crate::timer::RateTransform;
use crate::timer::TimeRate;

use super::storable::StorageTypeMap;
use super::timer::Time;
use super::timer::TimeMetric;
use super::timer::Timer;

pub trait Mobject:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type Entity: 'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MobjectEntity<M, S> {
    mobject: M,
    structure: S,
}

// pub trait SpatialMobject: Mobject {}

// pub trait TemporalMobject: Mobject {}

pub trait Worldline<TM>: Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
{
    // type Transformed<TR0>
    // where
    //     TR0: TimeRate;
    type Observatory;

    // fn init_presentation(
    //     &self,
    //     mobject: M,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     format: wgpu::TextureFormat,
    // ) -> Self::Presentation;
    // fn transform<TR0>(self, rate_transform: RateTransform<TR0, TM>) -> Self::Transformed<TR0>
    // where
    //     TR0: TimeRate;
    fn observe(&self, metric_time: TM::MetricTime) -> Self::Observatory;
}

pub trait Present {
    type Presentation;

    fn present(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Presentation;
}

pub trait Prepare<E> {
    // type Output;

    fn prepare(
        &mut self,
        entity: E,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ); // -> &Self::Output; // input presentation write-only
}

pub trait Render {
    fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}

// pub trait PreparableWorldline<M>: Worldline<M> {
//     type Presentation: Prepare<M>;

//     fn initialize(
//         &self,
//         mobject: &M,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Presentation;
//     fn prepare(
//         &self,
//         presentation: &mut Self::Presentation,
//         mobject: &M,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

// pub trait RenderableWorldline<M>: PreparableWorldline<M>
// where
//     Self::Observant: Prepare<M> + Render,
// {
// }

// pub trait AllocatedWorldline {
//     fn prepare(
//         &self,
//         time: Time,
//         time_interval: Range<f32>,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> PresentationKey<Self::Presentation>;
//     fn prepare(
//         &self,
//         time: Time,
//         time_interval: Range<f32>,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> PresentationKey<Self::Presentation>;
// }

// #[derive(serde::Deserialize, serde::Serialize)]
// pub struct Schedule<MI>
// where
//     MI: 'static,
// {
//     time_interval: std::ops::Range<Time>,
//     worldline: serde_traitobject::Box<dyn Worldline<Observatory = MI>>,
// }

// impl<M> Schedule<M> {
//     fn observe(&self, time: Time) -> Option<M> {
//         self.time_interval.contains(&time).then(|| {
//             self.worldline
//                 .observe(time, self.time_interval.clone().into())
//         })
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticWorldline<M>
where
    M: Mobject,
{
    // time_rate: TR,
    entity: Arc<M::Entity>,
}

impl<TM, M> Worldline<TM> for StaticWorldline<M>
where
    TM: TimeMetric,
    M: Mobject,
{
    // type Transformed<TR0> = StaticWorldline<RateChain<TR0, TR::TimeMetric, TR>, M>
    // where
    //     TR0: TimeRate;
    type Observatory = Arc<M::Entity>;

    // fn transform<TR0>(
    //     self,
    //     rate_transform: RateTransform<TR0, TR::TimeMetric>,
    // ) -> Self::Transformed<TR0>
    // where
    //     TR0: TimeRate,
    // {
    //     StaticWorldline {
    //         time_rate: RateChain {
    //             rate_transform,
    //             time_rate: self.time_rate,
    //         },
    //         entity: self.entity,
    //     }
    // }

    fn observe(&self, _metric_time: TM::MetricTime) -> Self::Observatory {
        self.entity.clone()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicWorldline<M, CU>
where
    M: Mobject,
{
    entity: M::Entity,
    continuous_update: CU,
}

impl<TM, M, CU> Worldline<TM> for DynamicWorldline<M, CU>
where
    TM: TimeMetric,
    M: Mobject,
    M::Entity: Clone,
    CU: ContinuousUpdate<TM, M::Entity>,
{
    // type Transformed<TR0> = DynamicWorldline<RateChain<TR0, TR::TimeMetric, TR>, M, CU>
    // where
    //     TR0: TimeRate;
    type Observatory = M::Entity;

    // fn transform<TR0>(
    //     self,
    //     rate_transform: RateTransform<TR0, TR::TimeMetric>,
    // ) -> Self::Transformed<TR0>
    // where
    //     TR0: TimeRate,
    // {
    //     DynamicWorldline {
    //         time_rate: RateChain {
    //             rate_transform,
    //             time_rate: self.time_rate,
    //         },
    //         entity: self.entity,
    //         continuous_update: self.continuous_update,
    //     }
    // }

    fn observe(&self, metric_time: TM::MetricTime) -> Self::Observatory {
        self.continuous_update
            .continuous_update(*metric_time, self.entity.clone())
        // self.time_rate.eval_time(time).map(|metric_time| {
        //     self.continuous_update
        //         .continuous_update(*metric_time, self.entity.clone())
        // })
    }
}

impl<TM, M, S> Worldline<TM> for MobjectEntity<Arc<M>, S>
where
    TM: TimeMetric,
    M: Mobject,
    S: Worldline<TM>,
{
    type Observatory = MobjectEntity<Arc<M>, S::Observatory>;

    fn observe(&self, metric_time: TM::MetricTime) -> Self::Observatory {
        MobjectEntity {
            mobject: self.mobject.clone(),
            structure: self.structure.observe(metric_time),
        }
    }
}

//

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0Structure {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0 {
    demo0: f32,
}

impl Mobject for MyMobject0 {
    type Entity = MobjectEntity<MyMobject0, MyMobject0Structure>;
}

impl<TM> Worldline<TM> for MyMobject0Structure
where
    TM: TimeMetric,
{
    type Observatory = MyMobject0Structure;

    #[allow(unused_variables)]
    fn observe(&self, time: Time, time_interval: Range<Time>) -> Self::Observatory {
        MyMobject0Structure {}
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1Structure<
    MI0 = MobjectEntity<MyMobject0, MyMobject0Structure>,
    MI1 = MobjectEntity<MyMobject0, MyMobject0Structure>,
> {
    m0: MI0,
    m1: MI1,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1 {
    demo1: f32,
}

impl Mobject for MyMobject1 {
    type Entity = MobjectEntity<MyMobject1, MyMobject1Structure>;
}

impl<MI0, MI1> Worldline for MyMobject1Structure<MI0, MI1>
where
    MI0: Worldline,
    MI1: Worldline,
{
    type Observatory = MyMobject1Structure<MI0::Observatory, MI1::Observatory>;

    fn observe(&self, time: Time, time_interval: Range<Time>) -> Self::Observatory {
        MyMobject1Structure {
            m0: self.m0.observe(time, time_interval),
            m1: self.m1.observe(time, time_interval),
        }
    }
}

// impl<TE, CU> Worldline for MySpatialMobjectDynamicWorldline<TE, CU>
// where
//     TE: TimeEval,
//     CU: ContinuousUpdate<TE::OutputTimeMetric, MySpatialMobject>,
// {
//     type Mobject = MySpatialMobject;

//     fn observe(
//         &self,
//         time: Time,
//         time_interval: Range<Time>,
//         mobject: Self::Mobject,
//     ) -> Self::Mobject {
//         self.continuous_update
//             .continuous_update(self.time_eval.time_eval(time, time_interval), mobject)
//     }
// }

// pub trait PreparableMobject: Mobject {
//     type Output: 'static + Send + Sync;

//     fn prepare(
//         &self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Output;
//     // fn present_in_place(
//     //     &self,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     format: wgpu::TextureFormat,
//     //     presentation: &mut Self::Presentation,
//     // ); // presentation write-only
// }

// pub trait UpdateOnce<PM>
// where
//     PM: PreparableMobject,
// {
//     fn update_once(&self, mobject: &mut PM);
// }

pub trait ContinuousUpdate<TM, E>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    fn continuous_update(&self, time: Time, entity: E) -> E;

    // {
    //     // let mut mobject = mobject.clone();
    //     // self.update(&mut mobject, time_metric);
    //     std::mem::replace(presentation, mobject.present(device, queue, format));
    // }
    // {
    //     mobject.present_in_place(device, queue, format, presentation);
    // }
}

// pub trait DiscreteUpdate {
//     fn discrete_update(self, timer: &Timer, alive: &Alive) -> &Alive;
// }

// pub struct Alive<'t, TQ, TS>
// where
//     TQ: TypeQuery,
//     TS: WorldlineState<TQ>,
// {
//     parent: P,
//     spawn_time: Rc<Time>,
//     // config: &'t Config,
//     // timer: &'t Timer,

//     // attached_mobject: AttachedMobject<'t, 'a, TQ>,
//     mobject: Arc<M>,
// }

// impl<U, PM> UpdateOnce<PM> for U
// where
//     U: Update<PM, NormalizedTimeMetric>,
//     PM: PreparableMobject,
// {
//     fn update_once(&self, mobject: &mut PM) {
//         self.update(mobject, NormalizedTimeMetric(1.0));
//     }
// }

// pub trait Prepare {
//     type Output;

//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Output;
// }

// pub trait Render {
//     fn render(
//         &self,
//         storage_type_map: &StorageTypeMap,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     );
// }
