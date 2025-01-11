use super::interpolate::Interpolate;

#[derive(Clone)]
pub struct Scalar(f32);

impl Scalar {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

// impl Interpolate for Scalar {
//     type Diff = f32;

//     fn interpolate(&self, diff: &Self::Diff, alpha: f32) -> Self {
//         Self(self.0 + diff * alpha)
//     }

//     // fn diff(&self, target: &Self) -> Self::Diff {
//     //     target.0 - self.0
//     // }

//     // fn is_negligible(diff: &Self::Diff) -> bool {
//     //     diff == &0.0
//     // }
// }
