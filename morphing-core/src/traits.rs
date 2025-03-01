use std::fmt::Debug;

use super::alive::AliveRoot;
use super::alive::Time;
use super::config::Config;
use super::renderable::AliveRenderable;
use super::renderable::LayerRenderableState;
use super::storage::Storable;
use super::timeline::AliveTimeline;
use super::timeline::CollapsedTimelineState;
use super::timeline::TimeMetric;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type MobjectPresentation: Storable;

    fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

// pub trait MobjectPresentation: Send + Sync + Any {
//     fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
// }

pub trait MobjectBuilder<L>
where
    L: Layer,
{
    type Instantiation: Mobject;

    fn instantiate(self, layer: &L, config: &Config) -> Self::Instantiation; // ?
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

pub trait Construct<L, M>: 'static
where
    L: Layer,
    M: Mobject,
{
    type OutputMobject: Mobject;

    fn construct<'a2, 'a1, 'a0>(
        self,
        root: &AliveRoot<'a0>,
        renderable: &AliveRenderable<'a1, 'a0, LayerRenderableState<L>>,
        timeline: AliveTimeline<'_, 'a1, 'a0, LayerRenderableState<L>, CollapsedTimelineState<M>>,
    ) -> AliveTimeline<
        'a2,
        'a1,
        'a0,
        LayerRenderableState<L>,
        CollapsedTimelineState<Self::OutputMobject>,
    >;
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

pub trait Layer:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type LayerPresentation: Storable;

    fn prepare(
        &self,
        time: Time,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::LayerPresentation;
    fn render(
        &self,
        layer_presentation: &Self::LayerPresentation,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

pub trait LayerBuilder {
    type Instantiation: Layer;

    fn instantiate(self, config: &Config) -> Self::Instantiation;
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
