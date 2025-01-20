use super::super::toplevel::world::World;

// pub trait VectorSpace: Clone + AddAssign + MulAssign<f32> {}

// impl<T> VectorSpace for T where T: Clone + AddAssign + MulAssign<f32> {}

pub trait Mobject: 'static + Clone {
    // type Diff: VectorSpace;

    // fn apply_diff(&self, diff: Self::Diff) -> Self;
    type Realization: MobjectRealization;

    fn realize(&self, device: &wgpu::Device) -> Self::Realization;
}

pub trait MobjectRealization: 'static {
    fn render(&self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass);
}

pub trait MobjectBuilder {
    type Instantiation: Mobject;

    fn instantiate(self, world: &World) -> Self::Instantiation;
}

// TODO: alive container morphisms

// #[derive(Clone)]
// struct LazyDiffField<T>(Option<T>);

// impl<T> AddAssign for LazyDiffField<T>
// where
//     T: VectorSpace,
// {
//     fn add_assign(&mut self, rhs: Self) {
//         if let Some(rhs) = rhs.0 {
//             if let Some(lhs) = self.0.as_mut() {
//                 *lhs += rhs;
//             } else {
//                 self.0 = Some(rhs);
//             }
//         }
//     }
// }

// impl<T> MulAssign<f32> for LazyDiffField<T>
// where
//     T: VectorSpace,
// {
//     fn mul_assign(&mut self, rhs: f32) {
//         if let Some(lhs) = self.0.as_mut() {
//             *lhs *= rhs;
//         }
//     }
// }

// #[derive(Clone)]
// pub struct EmptyMobjectDiff;

// impl AddAssign for EmptyMobjectDiff {
//     fn add_assign(&mut self, _rhs: Self) {}
// }

// impl MulAssign<f32> for EmptyMobjectDiff {
//     fn mul_assign(&mut self, _rhs: f32) {}
// }

// #[derive(Clone)]
// pub struct EmptyMobject;

impl Mobject for () {
    // type Diff = EmptyMobjectDiff;

    // fn apply_diff(&self, _diff: Self::Diff) -> Self {
    //     Self
    // }

    type Realization = ();

    fn realize(&self, _device: &wgpu::Device) -> Self::Realization {
        ()
    }
}

impl MobjectBuilder for () {
    type Instantiation = ();

    fn instantiate(self, _world: &World) -> Self::Instantiation {
        ()
    }
}

impl MobjectRealization for () {
    fn render(&self, _queue: &wgpu::Queue, _render_pass: &mut wgpu::RenderPass) {}
}

// trait HomogeneousMobject<M> where M: Mobject {
//     fn
// }
