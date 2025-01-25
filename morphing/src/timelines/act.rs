use std::fmt::Debug;

use super::super::mobjects::mobject::Mobject;

pub trait Act<M>: Clone
where
    M: Mobject,
{
    type Diff: MobjectDiff<M>;

    // fn diff(self, mobject: &M) -> Self::Diff;
    // fn apply_diff(mobject: &M, diff: Self::Diff) -> M;
    // fn scale_diff(alpha: f32, diff: &Self::Diff) -> Self::Diff;

    fn act(self, mobject: &M) -> Self::Diff;
}

pub trait MobjectDiff<M>:
    'static + Clone + Debug + serde::de::DeserializeOwned + serde::Serialize
where
    M: Mobject,
{
    fn apply(&self, mobject: &mut M, alpha: f32);
    fn apply_realization(
        &self,
        mobject_realization: &mut M::Realization,
        reference_mobject: &M,
        alpha: f32,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()>; // mobject_realization write-only
}
