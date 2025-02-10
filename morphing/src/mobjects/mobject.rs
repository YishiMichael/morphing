use std::fmt::Debug;

use super::super::toplevel::config::Config;

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

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    iced::widget::shader::wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: iced::widget::shader::wgpu::Operations {
                            load: iced::widget::shader::wgpu::LoadOp::Load,
                            store: iced::widget::shader::wgpu::StoreOp::Store,
                        },
                    },
                )],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );
        self.draw(&mut render_pass);
    }
}

pub trait MobjectBuilder {
    type Instantiation: Mobject;

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
