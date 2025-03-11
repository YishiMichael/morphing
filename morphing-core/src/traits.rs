use std::fmt::Debug;

use super::config::Config;
use super::stage::Layer;
use super::stage::World;
use super::storable::StorableKeyFn;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::MobjectLocate;
use super::timeline::MobjectLocated;
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
    ) -> Alive<'_, MobjectLocated<W, LI, Self::ChannelIndex, Self>, CollapsedTimelineState, SKF>
    where
        MobjectLocated<W, LI, Self::ChannelIndex, Self>: MobjectLocate,
        SKF: StorableKeyFn;
    fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

pub trait MobjectBuilder<L>
where
    L: Layer,
{
    type Instantiation: Mobject<L>;

    fn instantiate(self, config: &Config) -> Self::Instantiation;
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

pub trait Update<TM, ML>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
    ML: MobjectLocate,
{
    fn update(&self, time_metric: TM, mobject: &mut ML::Mobject);
    fn prepare_presentation(
        &self,
        time_metric: TM,
        mobject: &ML::Mobject,
        mobject_presentation: &mut <ML::Mobject as Mobject<ML::Layer>>::MobjectPresentation,
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

pub trait Construct<ML>: 'static
where
    ML: MobjectLocate,
{
    type OutputMobjectLocate: MobjectLocate;

    fn construct<'a, SKF>(
        self,
        world_attachment: &'a <ML::World as World>::Attachment<'a, SKF>,
        config: &Config,
        timer: &Timer,
        alive: Alive<'a, ML, CollapsedTimelineState, SKF>,
    ) -> Alive<'a, Self::OutputMobjectLocate, CollapsedTimelineState, SKF>
    where
        SKF: StorableKeyFn;
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
