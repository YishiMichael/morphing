use std::fmt::Debug;

use super::config::Config;
use super::stage::Layer;
use super::stage::LayerAttachment;
use super::stage::LayerIndex;
use super::stage::World;
use super::stage::WorldAttachment;
use super::timeline::Alive;
use super::timeline::CollapsedTimelineState;
use super::timeline::TypeQuery;
use super::timer::TimeMetric;
use super::timer::Timer;

pub trait Component: serde::de::DeserializeOwned + serde::Serialize {
    type ShaderTypes: ComponentShaderTypes;

    fn to_shader_types(&self) -> Self::ShaderTypes;
}

pub trait ComponentShaderTypes {
    type Buffers;

    fn bind_group_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout;
    fn bind_group_from_buffers(device: &wgpu::Device, buffers: &Self::Buffers) -> wgpu::BindGroup;
    fn new_buffers(&self, device: &wgpu::Device) -> Self::Buffers;
    fn new_buffers_initialized(&self, device: &wgpu::Device) -> Self::Buffers;
    fn write_buffers(&self, queue: &wgpu::Queue, buffers: &mut Self::Buffers);
}

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
}

pub trait Presentation<M>: 'static + Send + Sync
where
    M: Mobject,
{
    fn presentation(mobject: &M, device: &wgpu::Device) -> Self;
}

pub trait MobjectBuilder<L>
where
    L: Layer,
{
    type OutputTypeQuery<W, LI>: TypeQuery<World = W, LayerIndex = LI, Layer = L>
    where
        W: World,
        LI: LayerIndex<W, Layer = L>;

    fn instantiate<'t, 'a, W, LI>(
        self,
        layer_attachment_residue: &'a L::Residue<'t, W, LI>,
        config: &'t Config,
    ) -> Alive<'t, 'a, Self::OutputTypeQuery<W, LI>, CollapsedTimelineState>
    where
        W: World,
        LI: LayerIndex<W, Layer = L>;
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
        presentation: &mut TQ::Presentation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ); // presentation write-only
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
    fn construct<'t, 'a>(
        self,
        world_attachment: &'a WorldAttachment<'t, TQ::World, <TQ::World as World>::Residue<'t>>,
        config: &'t Config,
        timer: &'t Timer,
        alive: Alive<'t, 'a, TQ, CollapsedTimelineState>,
    ) -> Alive<'t, 'a, TQ, CollapsedTimelineState>;
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
