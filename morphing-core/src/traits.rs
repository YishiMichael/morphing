use std::fmt::Debug;

use super::config::Config;
use super::stage::ChannelIndex;
use super::stage::Layer;
use super::stage::World;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::CompatibleAttachment;
use super::timeline::MobjectQuery;
use super::timeline::TimeMetric;
use super::timeline::Timer;

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
    type ChannelIndex: ChannelIndex;
    type Mobject: Mobject;
    type MobjectPresentation: MobjectPresentation<Self::Mobject>;

    fn instantiate(self, config: &Config) -> Self::Mobject;
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

pub trait Update<TM, MQ>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
    MQ: MobjectQuery,
{
    fn update(&self, time_metric: TM, mobject: &mut MQ::Mobject);
    fn prepare_presentation(
        &self,
        time_metric: TM,
        mobject: &MQ::Mobject,
        mobject_presentation: &mut MQ::MobjectPresentation,
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

pub trait Construct<MQ>: 'static
where
    MQ: MobjectQuery,
{
    // type OutputMobjectQuery: MobjectQuery;

    fn construct<'a, CA>(
        self,
        world_attachment: &'a <MQ::World as World>::Attachment<'a, MQ::StorableKeyFn>,
        config: &'a Config,
        timer: &'a Timer,
        alive: Alive<'a, MQ, CA, CollapsedTimelineState>,
    ) -> Alive<'a, MQ, CA, CollapsedTimelineState>
    where
        CA: CompatibleAttachment<MQ>;
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
