use std::fmt::Debug;

use super::super::mobjects::mobject::Mobject;

pub trait Act<M>
where
    M: Mobject,
{
    type Diff: MobjectDiff<M>;

    // fn diff(self, mobject: &M) -> Self::Diff;
    // fn apply_diff(mobject: &M, diff: Self::Diff) -> M;
    // fn scale_diff(alpha: f32, diff: &Self::Diff) -> Self::Diff;

    fn act(self, mobject: &M) -> Self::Diff;
}

pub trait ApplyAct<M>
where
    M: Mobject,
{
    type Output<A>
    where
        A: Act<M>;

    fn apply_act<A>(self, act: A) -> Self::Output<A>
    where
        A: Act<M>;
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
    ); // mobject_realization write-only
}
