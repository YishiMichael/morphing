use super::super::mobjects::mobject::Mobject;

pub trait Update<M>: 'static + Clone + Send + Sync
where
    M: Mobject,
{
    fn update(&self, mobject: &mut M, alpha: f32);
    fn update_realization(
        &self,
        mobject_realization: &mut M::Realization,
        reference_mobject: &M,
        alpha: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()>; // mobject_realization write-only
}
