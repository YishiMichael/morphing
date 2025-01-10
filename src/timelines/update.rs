use super::super::mobjects::mobject::Mobject;

pub trait Update<M>: 'static
where
    M: Mobject,
{
    fn update(&self, mobject: &M, alpha: f32) -> M;
}
