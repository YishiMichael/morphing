use super::super::mobjects::mobject::Mobject;
use super::timeline::steady::SteadyTimeline;

pub trait Construct<M>: 'static
where
    M: Mobject,
{
    type Output: Mobject;

    fn construct(self, input: SteadyTimeline<M>) -> SteadyTimeline<Self::Output>;
}
