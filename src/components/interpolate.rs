pub(crate) trait Interpolate {
    type Diff;

    fn interpolate(&mut self, src: &Self, diff: &Self::Diff, alpha: f32);
    // fn diff(&self, target: &Self) -> Self::Diff;
    // fn is_negligible(diff: &Self::Diff) -> bool;
}

// pub(crate) trait InterpolatableField: Interpolate {
// }

// pub(crate) struct Interpolator<T: InterpolatableField> {
//     src: T,
//     diff: Option<T::Diff>,
// }

// impl<T: InterpolatableField> Interpolator<T> {
//     pub(crate) fn new(src: T, target: T) -> Self {
//         let diff = src.difference(&target);
//         Self {
//             src,
//             diff: (!T::is_negligible(&diff)).then_some(diff),
//         }
//     }

//     pub(crate) fn interpolate_zero(&mut self, dst: &mut T) {
//         if self.diff.is_none() {
//             std::mem::swap(dst, &mut self.src);
//         } // TODO: correct?
//     }

//     pub(crate) fn interpolate(&self, alpha: f32, dst: &mut T) {
//         if let Some(diff) = &self.diff {
//             dst.interpolate(&self.src, &diff, alpha);
//         }
//     }
// }
