use super::super::mobjects::mobject::Mobject;
use super::super::mobjects::mobject::MobjectDiff;

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
