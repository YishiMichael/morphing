pub trait Interpolate {
    type Output;

    fn interpolate(&self, alpha: f32) -> Self::Output;
    // fn diff(&self, target: &Self) -> Self::Diff;
    // fn is_negligible(diff: &Self::Diff) -> bool;
}

// pub trait InterpolatableField: Interpolate {
// }

// pub struct Interpolator<T: InterpolatableField> {
//     src: T,
//     diff: Option<T::Diff>,
// }

// impl<T: InterpolatableField> Interpolator<T> {
//     pub fn new(src: T, target: T) -> Self {
//         let diff = src.difference(&target);
//         Self {
//             src,
//             diff: (!T::is_negligible(&diff)).then_some(diff),
//         }
//     }

//     pub fn interpolate_zero(&mut self, dst: &mut T) {
//         if self.diff.is_none() {
//             std::mem::swap(dst, &mut self.src);
//         } // TODO: correct?
//     }

//     pub fn interpolate(&self, alpha: f32, dst: &mut T) {
//         if let Some(diff) = &self.diff {
//             dst.interpolate(&self.src, &diff, alpha);
//         }
//     }
// }
