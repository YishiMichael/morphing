use super::interpolate::{InterpolatableField, Interpolate};

#[derive(Clone)]
pub(crate) struct Scalar(f32);

impl Scalar {
    pub(crate) fn new(value: f32) -> Self {
        Self(value)
    }
}

impl Interpolate for Scalar {
    type Diff = Self;

    fn interpolate(&mut self, src: &Self, diff: &Self::Diff, alpha: f32) {
        self.0 = src.0 + diff.0 * alpha;
    }
}

impl InterpolatableField for Scalar {
    fn difference(&self, target: &Self) -> Self::Diff {
        Self(target.0 - self.0)
    }

    fn is_negligible(diff: &Self::Diff) -> bool {
        diff.0 == 0.0
    }
}
