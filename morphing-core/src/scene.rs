type Duration = std::time::Duration;
type Clock = std::time::SystemTime;

use std::sync::Arc;

type Signal = f32;
type Resource = (bool,);
type RenderContext = wgpu::RenderPass<'static>;

pub trait Lifecycle: 'static + Send + Sync {
    // type Signal = f32;
    // type Resource = (bool,);

    fn setup(&self) -> Resource;
    fn prepare(&self, signal: Signal, resource: &mut Resource);
    fn render(&self, resource: &Resource);
}

pub struct Supervisor<C> {
    time: f32,
    lifecycles: Vec<Box<dyn Lifecycle>>,
    config: C,
}

impl<C> Supervisor<C> {
    fn with<L>(&mut self, lifecycle: L)
    where
        L: Lifecycle,
    {
        self.lifecycles.push(Box::new(lifecycle))
    }
}
