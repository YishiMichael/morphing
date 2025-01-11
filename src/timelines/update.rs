use super::super::mobjects::mobject::Mobject;

pub trait Update<M>: 'static
where
    M: Mobject,
{
    fn update(&self, mobject: &M, alpha: f32) -> M;
}

pub trait ApplyUpdate<M, U>
where
    M: Mobject,
    U: Update<M>,
{
    type Output;

    fn apply_update(self, update: U) -> Self::Output;
}
