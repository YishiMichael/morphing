use super::super::mobjects::mobject::Mobject;
use super::alive::Alive;
use super::timeline::steady::SteadyTimeline;
use super::timeline::Supervisor;

pub trait Construct<M>: Clone
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
