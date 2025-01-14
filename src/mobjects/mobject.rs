use std::ops::AddAssign;
use std::ops::MulAssign;

use super::super::toplevel::renderer::Renderer;

pub trait VectorSpace: Clone + AddAssign + MulAssign<f32> {}

impl<T> VectorSpace for T where T: Clone + AddAssign + MulAssign<f32> {}

pub trait Mobject: 'static + Clone {
    type Diff: VectorSpace;

    // fn apply_diff(&self, diff: Self::Diff) -> Self;
    fn render(&self, renderer: &Renderer);
}

#[derive(Clone)]
struct LazyDiffField<T>(Option<T>);

impl<T> AddAssign for LazyDiffField<T>
where
    T: VectorSpace,
{
    fn add_assign(&mut self, rhs: Self) {
        if let Some(rhs) = rhs.0 {
            if let Some(lhs) = self.0.as_mut() {
                *lhs += rhs;
            } else {
                self.0 = Some(rhs);
            }
        }
    }
}

impl<T> MulAssign<f32> for LazyDiffField<T>
where
    T: VectorSpace,
{
    fn mul_assign(&mut self, rhs: f32) {
        if let Some(lhs) = self.0.as_mut() {
            *lhs *= rhs;
        }
    }
}

#[derive(Clone)]
pub struct EmptyMobjectDiff;

impl AddAssign for EmptyMobjectDiff {
    fn add_assign(&mut self, _rhs: Self) {}
}

impl MulAssign<f32> for EmptyMobjectDiff {
    fn mul_assign(&mut self, _rhs: f32) {}
}

#[derive(Clone)]
pub struct EmptyMobject;

impl Mobject for EmptyMobject {
    type Diff = EmptyMobjectDiff;

    fn apply_diff(&self, _diff: Self::Diff) -> Self {
        Self
    }

    fn render(&self, _renderer: &Renderer) {}
}
