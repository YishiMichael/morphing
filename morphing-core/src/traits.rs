use std::fmt::Debug;
use std::hash::Hash;

use super::config::Config;
use super::stage::Layer;
use super::stage::LayerIndexed;
use super::stage::PresentationChannel;
use super::stage::WorldIndexed;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::Locate;
use super::timeline::Located;
use super::timeline::TimeMetric;
use super::timeline::Timer;

pub trait Mobject<L>:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    L: Layer,
{
    type ChannelIndex;
    type MobjectPresentation: 'static + Send + Sync;

    fn spawn<W, LI, SKF>(
        self: Box<Self>,
        layer_architecture: &L::Architecture<SKF>,
    ) -> Alive<'_, Located<W, LI, Self::ChannelIndex, Self>, CollapsedTimelineState, SKF>
    where
        Located<W, LI, Self::ChannelIndex, Self>: Locate,
        SKF: StorableKeyFn;
    fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

// pub trait MobjectPresentation: Send + Sync + Any {
//     fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
// }

pub trait MobjectBuilder<L>
where
    L: Layer,
{
    type Instantiation: Mobject<L>;

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

pub trait Update<TM, L, M>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
    L: Layer,
    M: Mobject<L>,
{
    fn update(&self, time_metric: TM, mobject: &mut M);
    fn prepare_presentation(
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

pub trait Construct<W, L, M>: 'static
where
    W: WorldIndexed<Self::OutputLayerIndex>,
    L: Layer,
    M: Mobject<L>,
{
    type OutputLayerIndex;
    type OutputMobject: Mobject<<W as WorldIndexed<Self::OutputLayerIndex>>::Layer>;

    fn construct<'a, LI, CI, SKF>(
        self,
        world_attachment: &'a W::Attachment<'a, W::Architecture<SKF>>,
        config: &Config,
        timer: &Timer,
        mobject: Alive<'a, W, LI, CI, M, CollapsedTimelineState, SKF>,
    ) -> Alive<
        'a,
        W,
        Self::OutputLayerIndex,
        <Self::OutputMobject as Mobject<<W as WorldIndexed<Self::OutputLayerIndex>>::Layer>>::ChannelIndex,
        Self::OutputMobject,
        CollapsedTimelineState,
        SKF,
    >
    where
        W: WorldIndexed<LI, Layer = L>,
        L: LayerIndexed<CI, Channel = PresentationChannel<M::MobjectPresentation>>,
        <W as WorldIndexed<Self::OutputLayerIndex>>::Layer: LayerIndexed<<Self::OutputMobject as Mobject<<W as WorldIndexed<Self::OutputLayerIndex>>::Layer>>::ChannelIndex, Channel = PresentationChannel<<Self::OutputMobject as Mobject<<W as WorldIndexed<Self::OutputLayerIndex>>::Layer>>::MobjectPresentation>>,
        SKF: StorableKeyFn;
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

pub trait StorableKeyFn: 'static + Debug + Send + Sync {
    type Output: Clone + Eq + Hash + Send + Sync;

    fn eval_key<S>(serializable: &S) -> Self::Output
    where
        S: serde::Serialize;
}

// pub trait LayerBuilder {
//     type Instantiation: Layer;

//     fn instantiate(self, config: &Config) -> Self::Instantiation;
// }

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
