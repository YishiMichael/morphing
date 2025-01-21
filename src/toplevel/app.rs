use std::ffi::OsStr;
use std::ops::Range;
use std::path::PathBuf;
use std::process::Stdio;

use pollster::FutureExt;

use super::config::Config;
use super::config::VideoConfig;
use super::renderer::Renderer;
use super::scene::PresentationCollection;
use super::scene::Scene;
use super::settings::SceneSettings;
use super::settings::Settings;
use super::settings::VideoSettings;

#[derive(Clone, Copy)]
enum ProgressSpeed {
    ForwardX0_50,
    ForwardX0_75,
    ForwardX1_00,
    ForwardX1_25,
    ForwardX1_50,
    ForwardX2_00,
    BackwardX0_50,
    BackwardX0_75,
    BackwardX1_00,
    BackwardX1_25,
    BackwardX1_50,
    BackwardX2_00,
}

impl ProgressSpeed {
    fn value(&self) -> f32 {
        match self {
            Self::ForwardX0_50 => 0.50,
            Self::ForwardX0_75 => 0.75,
            Self::ForwardX1_00 => 1.00,
            Self::ForwardX1_25 => 1.25,
            Self::ForwardX1_50 => 1.50,
            Self::ForwardX2_00 => 2.00,
            Self::BackwardX0_50 => -0.50,
            Self::BackwardX0_75 => -0.75,
            Self::BackwardX1_00 => -1.00,
            Self::BackwardX1_25 => -1.25,
            Self::BackwardX1_50 => -1.50,
            Self::BackwardX2_00 => -2.00,
        }
    }

    fn display_str(&self) -> &'static str {
        match self {
            Self::ForwardX0_50 => "0.5x",
            Self::ForwardX0_75 => "0.75x",
            Self::ForwardX1_00 => "speed",
            Self::ForwardX1_25 => "1.25x",
            Self::ForwardX1_50 => "1.5x",
            Self::ForwardX2_00 => "2x",
            Self::BackwardX0_50 => "-0.5x",
            Self::BackwardX0_75 => "-0.75",
            Self::BackwardX1_00 => "-1x",
            Self::BackwardX1_25 => "-1.25",
            Self::BackwardX1_50 => "-1.5x",
            Self::BackwardX2_00 => "-2x",
        }
    }
}

struct Progress {
    time_interval: Range<f32>,
    // anchor_time: f32,
    // instant: Instant,
    time: f32,
    base_speed: f32,
    progress_speed: ProgressSpeed,
    paused: bool,
}

impl Progress {
    fn new(full_time: f32, base_speed: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            time: 0.0,
            // instant: Instant::now(),
            base_speed,
            progress_speed: ProgressSpeed::ForwardX1_00,
            paused: true,
        }
    }

    fn progress_speed(&self) -> ProgressSpeed {
        self.progress_speed
    }

    fn paused(&self) -> bool {
        self.paused
    }

    // fn get_time(&mut self) -> f32 {
    //     let mut time = self.anchor_time + self.instant.elapsed().as_secs_f32() * self.speed();
    //     if !self.time_interval.contains(&time) {
    //         time = time.clamp(self.time_interval.start, self.time_interval.end);
    //         self.progress_speed = 0;
    //         self.anchor_time = time;
    //         self.instant = Instant::now();
    //     }
    //     time
    // }

    fn forward_time(&mut self, app_delta_time: f32) -> f32 {
        if !self.paused {
            self.time += app_delta_time * self.base_speed * self.progress_speed.value();
            if !self.time_interval.contains(&self.time) {
                self.paused = true;
                self.time = self
                    .time
                    .clamp(self.time_interval.start, self.time_interval.end);
            }
        }
        self.time
    }

    fn set_time(&mut self, time: f32) -> f32 {
        self.time = time;
        time
    }

    fn set_progress_speed(&mut self, progress_speed: ProgressSpeed) {
        self.progress_speed = progress_speed;
    }

    fn play_or_pause(&mut self) {
        self.paused = !self.paused;
    }
}

pub(crate) struct App {
    settings: Settings,
    progress: Progress,
    scenes: Vec<Box<dyn Scene>>,
    // window: Option<Arc<winit::window::Window>>,
    // renderer: OnceLock<Renderer>,
    // progress: Progress,
    // control_pressed: bool,
    // presentation_collection: Option<PresentationCollection>,
}

impl App {
    pub fn new(// presentation_collection: PresentationCollection,
        // window_config: WindowConfig,
        // video_config: VideoConfig,
        // config: Config,
    ) -> Self {
        // env_logger::init();
        // let event_loop = winit::event_loop::EventLoop::new().unwrap();
        Self {
            // window_config,
            // video_config,
            state: None,
            // renderer: OnceLock::new(),
            // progress: Progress::new(presentation_collection.full_time()),
            // control_pressed: false,
            // presentation_collection: None,
        }
        // event_loop.run_app(&mut app)
    }

    fn render(&self, time: f32) {
        self.presentation_collection
            .present_all(time, self.renderer.get().unwrap())
    }

    fn on_redraw_requested(&mut self) {
        if self.progress.speed_level != 0 {
            let time = self.progress.get_time();
            self.render(time);
        }
    }

    fn on_key_down(&mut self, key: winit::keyboard::Key, control_pressed: bool) {
        match key {
            winit::keyboard::Key::Named(named_key) => match named_key {
                winit::keyboard::NamedKey::ArrowRight if !control_pressed => {
                    let time = self
                        .progress
                        .forward_time(self.window_config.forward_seconds);
                    self.render(time);
                }
                winit::keyboard::NamedKey::ArrowRight if control_pressed => {
                    let time = self
                        .progress
                        .forward_time(self.window_config.fast_forward_seconds);
                    self.render(time);
                }
                winit::keyboard::NamedKey::ArrowLeft if !control_pressed => {
                    let time = self
                        .progress
                        .forward_time(-self.window_config.forward_seconds);
                    self.render(time);
                }
                winit::keyboard::NamedKey::ArrowLeft if control_pressed => {
                    let time = self
                        .progress
                        .forward_time(-self.window_config.fast_forward_seconds);
                    self.render(time);
                }
                winit::keyboard::NamedKey::ArrowUp => {
                    let time = self
                        .progress
                        .set_speed_level(|speed_level| speed_level.max(0) + 1);
                    self.render(time);
                }
                winit::keyboard::NamedKey::ArrowDown => {
                    let time = self
                        .progress
                        .set_speed_level(|speed_level| speed_level.min(0) - 1);
                    self.render(time);
                }
                winit::keyboard::NamedKey::Space => {
                    let time = self
                        .progress
                        .set_speed_level(|speed_level| if speed_level != 0 { 0 } else { 1 });
                    self.render(time);
                }
                _ => {}
            },
            winit::keyboard::Key::Character(ch) => match ch.as_str() {
                "s" if control_pressed => {
                    let time = self.progress.set_speed_level(|_| 0);
                    self.render(time);
                    if let Some(save_file) = rfd::AsyncFileDialog::new()
                        .add_filter("MP4", &["mp4"])
                        .add_filter("PNG", &["png"])
                        .save_file()
                        .block_on()
                    {
                        let path: PathBuf = save_file.into();
                        match path.extension().map(OsStr::to_str).flatten() {
                            Some("mp4") => self.save_video(path),
                            Some("png") => self.save_image(path),
                            _ => panic!("Unsupported output file extension: {path:?}"),
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn save_video(&self, path: PathBuf) {
        let mut ffmpeg = essi_ffmpeg::FFmpeg::new()
            .stderr(Stdio::inherit())
            .input_with_file("-".into())
            .done()
            .output_as_file(path)
            .done()
            .start()
            .unwrap();

        let texture = self.renderer.get().unwrap().create_texture();
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let full_time = self.progress.time_interval.end;
        let fps = self.video_config.fps;
        // (0..=(full_time / fps).ceil() as u32).for_each(|i| i as f32 * fps)
        // ffmpeg.stdin()
    }

    fn save_image(&self, path: PathBuf) {
        todo!()
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.renderer.get_or_init(|| {
            let window = event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_inner_size::<winit::dpi::PhysicalSize<u32>>(
                            winit::dpi::PhysicalSize::from(self.window_config.size),
                        ),
                )
                .unwrap();
            let renderer = Renderer::new(window).unwrap();
            self.progress.set_base_speed(self.window_config.base_speed);
            self.progress.set_speed_level(|_| 1);
            renderer
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                self.on_redraw_requested();
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.control_pressed = modifiers.state().control_key();
            }
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key,
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.on_key_down(logical_key, self.control_pressed);
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.renderer.get().unwrap().request_redraw();
    }
}
