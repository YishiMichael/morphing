use super::super::mobjects::mobject::Mobject;
use super::timeline::steady::SteadyTimeline;

pub trait Construct<T>
where
    T: Mobject,
{
    type Input: Mobject;
    type Output: Mobject;

    fn construct(self, input: SteadyTimeline<Self::Input>) -> SteadyTimeline<Self::Output>;
}
