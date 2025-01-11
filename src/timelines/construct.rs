use super::super::mobjects::mobject::Mobject;
use super::timeline::steady::SteadyTimeline;
use super::timeline::Alive;
use super::timeline::Supervisor;

pub trait Construct<M>: 'static
where
    M: Mobject,
{
    type Output: Mobject;

    fn construct<'a>(
        self,
        input: Alive<'a, SteadyTimeline<M>>,
        sv: &Supervisor,
    ) -> Alive<'a, SteadyTimeline<Self::Output>>;
}
