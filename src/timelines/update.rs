use super::super::mobjects::mobject::Mobject;

pub trait Update<M>: 'static
where
    M: Mobject,
{
    fn update(&self, mobject: &M, alpha: f32) -> M;
}

pub trait ApplyUpdate<M>
where
    M: Mobject,
{
    type Output<U>
    where
        U: Update<M>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<M>;
}
