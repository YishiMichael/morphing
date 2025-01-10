use super::super::mobjects::mobject::Mobject;

pub trait ApplyAct<M, A>
where
    M: Mobject,
    A: Act<M>,
{
    type Output;

    fn apply_act(self, act: A) -> Self::Output;
}

pub trait Act<M>
where
    M: Mobject,
{
    fn act(self, mobject: &mut M);
}
