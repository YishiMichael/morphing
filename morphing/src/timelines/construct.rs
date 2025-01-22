use super::super::mobjects::mobject::Mobject;
use super::super::toplevel::scene::Supervisor;
use super::alive::Alive;
use super::timeline::steady::SteadyTimeline;

pub trait Construct<M>
where
    M: Mobject,
{
    type Output: Mobject;

    fn construct<'a>(
        self,
        input: Alive<'a, SteadyTimeline<M>>,
        supervisor: &Supervisor,
    ) -> Alive<'a, SteadyTimeline<Self::Output>>;
}

pub trait ApplyConstruct<M>
where
    M: Mobject,
{
    type Output<C>
    where
        C: Construct<M>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<M>;
}
