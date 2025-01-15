use super::super::mobjects::mobject::Mobject;

pub trait Diff<M>: 'static + Clone
where
    M: Mobject,
{
    fn apply(self, mobject: &mut M);
    fn apply_partial(&self, mobject: &mut M, alpha: f32);
}

#[derive(Clone)]
pub struct ComposeDiff<D0, D1>(pub(crate) D0, pub(crate) D1);

impl<M, D0, D1> Diff<M> for ComposeDiff<D0, D1>
where
    M: Mobject,
    D0: Diff<M>,
    D1: Diff<M>,
{
    fn apply(self, mobject: &mut M) {
        self.1.apply(mobject);
        self.0.apply(mobject);
    }

    fn apply_partial(&self, mobject: &mut M, alpha: f32) {
        self.1.apply_partial(mobject, alpha);
        self.0.apply_partial(mobject, alpha);
    }
}

pub trait Act<M>
where
    M: Mobject,
{
    type Diff: Diff<M>;

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
