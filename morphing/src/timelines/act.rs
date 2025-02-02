use std::fmt::Debug;

use super::super::mobjects::mobject::Mobject;

pub trait Act<M>: Clone
where
    M: Mobject,
{
    type Diff: MobjectDiff<M>;

    fn act(self, mobject: &M) -> Self::Diff;
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
