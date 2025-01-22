use std::fmt::Debug;

use super::super::mobjects::mobject::Mobject;

pub trait Update<M>: 'static + Debug + serde::de::DeserializeOwned + serde::Serialize
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
    ); // mobject_realization write-only
}

pub trait ApplyUpdate<M>
where
    M: Mobject,
{
    type Output<U>
    where
        U: Update<M>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<M>;
}
