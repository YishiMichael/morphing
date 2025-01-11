use super::super::mobjects::mobject::Mobject;

pub trait Act<M>
where
    M: Mobject,
{
    // type Diff;

    // fn diff(self, mobject: &M) -> Self::Diff;
    // fn apply_diff(mobject: &M, diff: Self::Diff) -> M;
    // fn scale_diff(alpha: f32, diff: &Self::Diff) -> Self::Diff;

    fn act(self, mobject: &M) -> M::Diff;
}

pub trait ApplyAct<M, A>
where
    M: Mobject,
    A: Act<M>,
{
    type Output;

    fn apply_act(self, act: A) -> Self::Output;
}
