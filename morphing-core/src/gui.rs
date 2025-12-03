use super::link::ChapterSymbol;
use std::sync::Arc;

pub struct MyApp {
    chapter_symbol: ChapterSymbol,
    clock: Clock,
    lifecycles: Arc<[Box<dyn Lifecycle>]>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, chapter_path: &str) -> Self {
        let mut supervisor = Supervisor {
            time: 0.0,
            lifecycles: Vec::new(),
            config: MyConfig {},
        };
        my_scene(&mut supervisor);
        Self {
            chapter_symbol: call_entrypoint(chapter_path),
            clock: Clock::now(),
            lifecycles: supervisor.lifecycles.into(),
        }
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _response) =
            ui.allocate_exact_size(egui::Vec2::new(960.0, 540.0), egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            SceneSlice {
                time: self.clock.elapsed().unwrap().as_secs_f32(),
                lifecycles: self.lifecycles.clone(),
            },
        ));
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.custom_painting(ui);
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Morphing App",
        Default::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, "../examples/hello_morphing")))),
    )
}
