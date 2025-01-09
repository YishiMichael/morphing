use super::super::mobjects::mobject::Mobject;

pub trait Act<T>
where
    T: Mobject,
{
    fn act(self, mobject: &mut T);
}
