use std::fmt::Debug;

use super::config::Config;
use super::stage::Layer;
use super::stage::LayerAttachment;
use super::stage::LayerIndex;
use super::stage::World;
use super::stage::WorldAttachment;
use super::stage::WorldIndexed;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::TimeMetric;
use super::timeline::Timer;
use super::timeline::TypeQuery;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    // type MobjectPresentation: 'static + Send + Sync;

    // fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

pub trait MobjectPresentation<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn presentation(mobject: &M, device: &wgpu::Device) -> Self;
}

pub trait MobjectBuilder<L>
where
    L: Layer,
{
    // type ChannelIndex: ChannelIndex;
    type OutputTypeQuery<W, LI>: TypeQuery<World = W, LayerIndex = LI, Layer = L>;
    // type Mobject: Mobject;
    // type MobjectPresentation: MobjectPresentation<Self::Mobject>;

    fn instantiate<'a, W, LI>(
        self,
        layer_attachment: &'a LayerAttachment<'a, W, LI, L, L::Residue<'a, W, LI>>,
        config: &'a Config,
    ) -> Alive<'a, Self::OutputTypeQuery<W, LI>, CollapsedTimelineState>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex;
    // fn spawn<W, LI, SKF>(
    //     mobject: Box<Self::Mobject>,
    //     layer_architecture: &L::Architecture<SKF>,
    // ) -> Alive<
    //     '_,
    //     MobjectQueried<W, LI, Self::ChannelIndex, Self::Mobject>,
    //     CollapsedTimelineState,
    //     SKF,
    // >
    // where
    //     MobjectQueried<W, LI, Self::ChannelIndex, Self::Mobject>: MobjectQuery,
    //     SKF: StorableKeyFn;
}

pub trait Rate<TM>:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
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

pub trait Update<TM, TQ>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
    TQ: TypeQuery,
{
    fn update(&self, time_metric: TM, mobject: &mut TQ::Mobject);
    fn prepare_presentation(
        &self,
        time_metric: TM,
        mobject: &TQ::Mobject,
        mobject_presentation: &mut TQ::MobjectPresentation,
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

pub trait Construct<TQ>: 'static
where
    TQ: TypeQuery,
{
    type OutputTypeQuery: TypeQuery<World = TQ::World>;

    fn construct<'a>(
        self,
        world_attachment: &'a WorldAttachment<'a, TQ::World, <TQ::World as World>::Residue<'a>>,
        config: &'a Config,
        timer: &'a Timer,
        alive: Alive<'a, TQ, CollapsedTimelineState>,
    ) -> Alive<'a, TQ, CollapsedTimelineState>;
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
