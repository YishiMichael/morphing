pub(crate) trait Interpolate {
    type Diff;

    fn interpolate(&mut self, src: &Self, diff: &Self::Diff, alpha: f32);
}

pub(crate) trait InterpolatableField: Interpolate {
    fn difference(&self, target: &Self) -> Self::Diff;
    fn is_negligible(_diff: &Self::Diff) -> bool {
        false
    }
}

pub(crate) enum Interpolator<T: InterpolatableField> {
    Static(T),
    Dynamic(T, T::Diff),
}

impl<T: InterpolatableField> Interpolator<T> {
    pub(crate) fn new(source: T, target: T) -> Self {
        let diff = source.difference(&target);
        if T::is_negligible(&diff) {
            Self::Static(source)
        } else {
            Self::Dynamic(source, diff)
        }
    }

    pub(crate) fn interpolate_zero(&mut self, dst: &mut T) {
        if let Self::Static(source) = self {
            std::mem::swap(dst, source);
        }
    }

    pub(crate) fn interpolate(&self, alpha: f32, dst: &mut T) {
        if let Self::Dynamic(source, diff) = self {
            dst.interpolate(source, diff, alpha);
        }
    }
}
