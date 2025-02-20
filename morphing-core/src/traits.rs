use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use super::config::Config;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::DynamicTimelineId;
use super::timeline::StaticTimelineId;
use super::timeline::Supervisor;
use super::timeline::TimeMetric;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type MobjectPresentation: MobjectPresentation;

    fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

pub trait MobjectPresentation: Send + Sync + Any {
    fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}

// pub trait MobjectDiff<M>:
//     'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
// where
//     M: Mobject,
// {
//     fn apply(&self, mobject: &mut M, t: f32);
//     fn apply_presentation(
//         &self,
//         mobject_presentation: &mut M::MobjectPresentation,
//         reference_mobject: &M,
//         t: f32,
//         queue: &wgpu::Queue,
//     ); // mobject_presentation write-only
// }

pub trait MobjectBuilder {
    type Instantiation: Mobject;

    fn instantiate(self, config: &Config) -> Self::Instantiation;
}

pub trait Rate<TM>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
{
    type OutputTimeMetric: TimeMetric;

    fn eval(&self, time_metric: TM) -> Self::OutputTimeMetric;
}

pub trait IncreasingRate<TM>: Rate<TM>
where
    TM: TimeMetric,
{
}

pub trait Update<TM, M>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
    M: Mobject,
{
    fn update(&self, time_metric: TM, mobject: &mut M);
    fn update_presentation(
        &self,
        time_metric: TM,
        mobject: &M,
        mobject_presentation: &mut M::MobjectPresentation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ); // mobject_presentation write-only
}

// pub trait Act<TM, M>: Clone
// where
//     TM: TimeMetric,
//     M: Mobject,
// {
//     type Update: Update<TM, M>;

//     fn act(self, mobject: &M) -> Self::Update;
// }

pub trait Construct<M>: 'static + Clone
where
    M: Mobject,
{
    type OutputMobject: Mobject;

    fn construct<'sv, 'c, 's, S>(
        self,
        input: Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>,
        supervisor: &'sv Supervisor<'c, 's, S>,
    ) -> Alive<'sv, 'c, 's, S, CollapsedTimelineState<Self::OutputMobject>>
    where
        S: Storage;
}

// pub trait SerdeMobject: serde_traitobject::Deserialize + serde_traitobject::Serialize {}

// struct SerdeMobjectWrapper<M>(M);

// impl<M> SerdeMobject for M where M: Mobject {}

// pub trait SerdeUpdate: serde_traitobject::Deserialize + serde_traitobject::Serialize {}

// struct SerdeUpdateWrapper<U>(U);

// impl<U, TM, M> SerdeUpdate for SerdeUpdateWrapper<U>
// where
//     U: Update<TM, M>,
//     TM: TimeMetric,
// {
// }

pub trait Storage: 'static {
    fn static_allocate(&self, mobject: &dyn serde_traitobject::Serialize) -> StaticTimelineId;
    fn static_get(&self, id: &StaticTimelineId) -> Option<&Arc<dyn MobjectPresentation>>; // TODO: operate closure?
    fn static_set(
        &mut self,
        id: &StaticTimelineId,
        activate: Option<()>,
    ) -> Option<&mut Option<Arc<dyn MobjectPresentation>>>;
    fn dynamic_allocate(
        &self,
        mobject: &dyn serde_traitobject::Serialize,
        update: &dyn serde_traitobject::Serialize,
    ) -> DynamicTimelineId;
    fn dynamic_get(&self, id: &DynamicTimelineId) -> Option<&Box<dyn MobjectPresentation>>; // TODO: operate closure?
    fn dynamic_set(
        &mut self,
        id: &DynamicTimelineId,
        activate: Option<()>,
    ) -> Option<&mut Option<Box<dyn MobjectPresentation>>>;
}

// TODO: alive container morphisms

// #[derive(Clone)]
// struct LazyDiffField<T>(Option<T>);

// impl<T> AddAssign for LazyDiffField<T>
// where
//     T: VectorSpace,
// {
//     fn add_assign(&mut self, rhs: Self) {
//         if let Some(rhs) = rhs.0 {
//             if let Some(lhs) = self.0.as_mut() {
//                 *lhs += rhs;
//             } else {
//                 self.0 = Some(rhs);
//             }
//         }
//     }
// }

// impl<T> MulAssign<f32> for LazyDiffField<T>
// where
//     T: VectorSpace,
// {
//     fn mul_assign(&mut self, rhs: f32) {
//         if let Some(lhs) = self.0.as_mut() {
//             *lhs *= rhs;
//         }
//     }
// }

// #[derive(Clone)]
// pub struct EmptyMobjectDiff;

// impl AddAssign for EmptyMobjectDiff {
//     fn add_assign(&mut self, _rhs: Self) {}
// }

// impl MulAssign<f32> for EmptyMobjectDiff {
//     fn mul_assign(&mut self, _rhs: f32) {}
// }

// #[derive(Clone)]
// pub struct EmptyMobject;

// impl Mobject for () {
//     type Realization = ();

//     fn realize(&self, _device: &wgpu::Device) -> Self::Realization {
//         ()
//     }
// }

// impl MobjectBuilder for () {
//     type Instantiation = ();

//     fn instantiate(self, _world: &World) -> Self::Instantiation {
//         ()
//     }
// }

// impl MobjectRealization for () {
//     fn render(&self, _render_pass: &mut wgpu::RenderPass) {}
// }
