use std::fmt::Debug;

use super::config::Config;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::Supervisor;
use super::timeline::TimeMetric;

pub trait Mobject:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    type MobjectPresentation: MobjectPresentation;

    fn presentation(&self, device: &wgpu::Device) -> Self::MobjectPresentation;
}

pub trait MobjectPresentation: 'static + Send + Sync {
    fn render(&self, command_encoder: &mut wgpu::CommandEncoder, texture_view: &wgpu::TextureView);
}

// pub trait MobjectDiff<M>:
//     'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
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
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
where
    TM: TimeMetric,
{
    type OutputMetric: TimeMetric;

    fn eval(&self, t: f32) -> f32;
}

pub trait IncreasingRate<TM>: Rate<TM>
where
    TM: TimeMetric,
{
}

pub trait Act<M, TM>: Clone
where
    M: Mobject,
    TM: TimeMetric,
{
    type Update: Update<M, TM>;

    fn act(self, mobject: &M) -> Self::Update;
}

pub trait Update<M, TM>:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
where
    M: Mobject,
    TM: TimeMetric,
{
    fn update(&self, mobject: &mut M, t: f32);
    fn update_presentation(
        &self,
        mobject_presentation: &mut M::MobjectPresentation,
        reference_mobject: &M,
        t: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ); // mobject_presentation write-only
}

pub trait Construct<S, M>: Clone
where
    S: Storage,
    M: Mobject,
{
    type Output: Mobject;

    fn construct<'sv, 'c, 's>(
        self,
        input: Alive<'sv, 'c, 's, S, CollapsedTimelineState<M>>,
        supervisor: &'sv Supervisor<'c, 's, S>,
    ) -> Alive<'sv, 'c, 's, S, CollapsedTimelineState<Self::Output>>;
}

pub trait Storage: 'static {
    type Key;

    fn generate_key<KI>(&self, key_input: &KI) -> Self::Key
    where
        KI: serde::Serialize;
    fn get_unwrap<T>(&self, key: &Self::Key) -> &T; // TODO: operate closure?
    fn get_mut_or_insert<T, F>(&mut self, key: &Self::Key, f: F) -> &mut T
    where
        F: FnOnce() -> T;
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
