use std::fmt::Debug;

use super::super::mobjects::mobject::Mobject;

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
