use std::ops::Range;
use std::time::Instant;

use super::config::VideoConfig;
use super::config::WindowConfig;
use super::renderer::Renderer;
use super::scene::PresentationCollection;

struct Progress {
    time_interval: Range<f32>,
    anchor_time: f32,
    instant: Instant,
    base_speed: f32,
    speed_level: i32,
}

impl Progress {
    fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            anchor_time: 0.0,
            instant: Instant::now(),
            base_speed: 1.0,
            speed_level: 0,
        }
    }

    fn speed(&self) -> f32 {
        self.base_speed
            * match self.speed_level {
                0 => 0.0,
                exponent @ 0.. => 2.0_f32.powi(exponent - 1),
                exponent @ ..0 => -2.0_f32.powi(-exponent - 1),
            }
    }

    fn get_time(&mut self) -> f32 {
        let mut time = self.anchor_time + self.instant.elapsed().as_secs_f32() * self.speed();
        if !self.time_interval.contains(&time) {
            time = time.clamp(self.time_interval.start, self.time_interval.end);
            self.speed_level = 0;
            self.anchor_time = time;
            self.instant = Instant::now();
        }
        time
    }

    fn forward_time(&mut self, delta_time: f32) -> f32 {
        let mut time = self.get_time() + delta_time;
        if !self.time_interval.contains(&time) {
            time = time.clamp(self.time_interval.start, self.time_interval.end);
            self.speed_level = 0;
        }
        self.anchor_time = time;
        self.instant = Instant::now();
        time
    }

    fn set_base_speed(&mut self, base_speed: f32) -> f32 {
        let time = self.get_time();
        self.anchor_time = time;
        self.instant = Instant::now();
        self.base_speed = base_speed;
        time
    }

    fn set_speed_level<F>(&mut self, f: F) -> f32
    where
        F: FnOnce(i32) -> i32,
    {
        let time = self.get_time();
        self.anchor_time = time;
        self.instant = Instant::now();
        self.speed_level = f(self.speed_level);
        time
    }
}

pub(crate) struct App {
    window_config: WindowConfig,
    video_config: VideoConfig,
    renderer: Option<Renderer>,
    progress: Progress,
    control_pressed: bool,
    presentation_collection: PresentationCollection,
}

impl App {
    pub(crate) fn instantiate_and_run(
        presentation_collection: PresentationCollection,
        window_config: WindowConfig,
        video_config: VideoConfig,
    ) -> Result<(), winit::error::EventLoopError> {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut app = Self {
            window_config,
            video_config,
            renderer: None,
            progress: Progress::new(presentation_collection.full_time()),
            control_pressed: false,
            presentation_collection,
        };
        event_loop.run_app(&mut app)
    }

    fn render(&self, time: f32) {
        self.presentation_collection
            .present_all(time, self.renderer.as_ref().unwrap())
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
                    let save_path = rfd::FileDialog::new()
                        .add_filter("MP4", &["mp4"])
                        .add_filter("PNG", &["png"])
                        .save_file();
                    dbg!(save_path);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.renderer.is_none() {
            let window = event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_inner_size::<winit::dpi::PhysicalSize<u32>>(
                            winit::dpi::PhysicalSize::from(self.window_config.size),
                        ),
                )
                .unwrap();
            self.renderer = Some(Renderer::new(window).unwrap());
            self.progress.set_base_speed(self.window_config.base_speed);
            self.progress.set_speed_level(|_| 1);
        }
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
        self.renderer.as_ref().unwrap().request_redraw();
    }
}
