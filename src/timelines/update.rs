use super::super::mobjects::mobject::Mobject;

pub trait Update<T>
where
    T: Mobject,
{
    fn update(self, mobject: &T, alpha: f32);
}
