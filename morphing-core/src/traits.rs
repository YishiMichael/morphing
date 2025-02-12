use std::fmt::Debug;

use super::config::Config;
use super::timeline::Alive;
use super::timeline::SteadyTimeline;
use super::timeline::Supervisor;

pub trait Mobject:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    type MobjectPresentation: MobjectPresentation;

    fn presentation(
        &self,
        device: &iced::widget::shader::wgpu::Device,
    ) -> Self::MobjectPresentation;
}

pub trait MobjectPresentation: 'static + Send + Sync {
    fn draw<'rp>(&'rp self, render_pass: &mut iced::widget::shader::wgpu::RenderPass<'rp>);
}

pub trait MobjectDiff<M>:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
where
    M: Mobject,
{
    fn apply(&self, mobject: &mut M, alpha: f32);
    fn apply_presentation(
        &self,
        mobject_presentation: &mut M::MobjectPresentation,
        reference_mobject: &M,
        alpha: f32,
        queue: &iced::widget::shader::wgpu::Queue,
    ); // mobject_presentation write-only
}

pub trait MobjectBuilder {
    type Instantiation: Mobject;

    fn instantiate(self, config: &Config) -> Self::Instantiation;
}

pub trait Rate:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    fn eval(&self, t: f32) -> f32;
}

pub trait Act<M>: Clone
where
    M: Mobject,
{
    type Diff: MobjectDiff<M>;

    fn act(self, mobject: &M) -> Self::Diff;
}

pub trait Update<M>:
    'static + Clone + Send + Sync + Debug + serde::de::DeserializeOwned + serde::Serialize
where
    M: Mobject,
{
    fn update(&self, mobject: &mut M, alpha: f32);
    fn update_presentation(
        &self,
        mobject_presentation: &mut M::MobjectPresentation,
        reference_mobject: &M,
        alpha: f32,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
    ); // mobject_presentation write-only
}

pub trait Construct<M>: Clone
where
    M: Mobject,
{
    type Output: Mobject;

    fn construct<'c>(
        self,
        input: Alive<'c, SteadyTimeline<M>>,
        supervisor: &Supervisor,
    ) -> Alive<'c, SteadyTimeline<Self::Output>>;
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
