use super::interpolate::Interpolate;

#[derive(Clone)]
pub struct Scalar(f32);

impl Scalar {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl Interpolate for Scalar {
    type Diff = f32;

    fn interpolate(&mut self, src: &Self, diff: &Self::Diff, alpha: f32) {
        self.0 = src.0 + diff * alpha;
    }

    // fn diff(&self, target: &Self) -> Self::Diff {
    //     target.0 - self.0
    // }

    // fn is_negligible(diff: &Self::Diff) -> bool {
    //     diff == &0.0
    // }
}
