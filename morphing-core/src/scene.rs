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

struct SceneSlice {
    time: f32,
    lifecycles: Arc<[Box<dyn Lifecycle>]>,
}

impl egui_wgpu::CallbackTrait for SceneSlice {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // let (lifecycles, duration) = self;
        // for lifecycle in lifecycles.iter() {
        //     let resource_hashmap: &mut HashMap<TriangleLifecycle, Resource> =
        //         callback_resources.entry().or_insert_with(HashMap::new);
        //     let resource = resource_hashmap.entry()
        // }
        Vec::new()
    }

    fn paint(
        &self,
        info: epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {

        // let code = /*toml*/ r#"
        //     [a.b.c]
        //     d = 32
        // "#;
        // let code = /*typst*/ r#"
        //     #set page(numbering: "1")
        //     #set par(spacing: 1em)
        // "#;
        // let code = /*typst.math*/ r#"
        //     $a a = b$
        // "#;
    }
}
