use std::ops::Range;
use std::time::Instant;

use super::renderer::Renderer;
use super::scene::SupervisorData;

struct TimerSpeedLevel(i32);

impl TimerSpeedLevel {
    fn as_multiplier(&self) -> f32 {
        match self.0 {
            0 => 0.0,
            exponent @ 0.. => 2.0_f32.powi(exponent - 1),
            exponent @ ..0 => -2.0_f32.powi(-exponent - 1),
        }
    }

    fn accelerate(&mut self) {
        self.0 = self.0.max(0) + 1;
    }

    fn deaccelerate(&mut self) {
        self.0 = self.0.min(0) - 1;
    }

    fn pause(&mut self) {
        self.0 = 0;
    }

    fn is_paused(&self) -> bool {
        self.0 == 0
    }
}

struct AccumulativeTimer {
    time_interval: Range<f32>,
    base_speed: f32,
    anchor_time: f32,
    instant: Instant,
    speed_level: i32,
}

impl AccumulativeTimer {
    fn new(full_time: f32, base_speed: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            base_speed,
            anchor_time: 0.0,
            instant: Instant::now(),
            speed_level: 0,
        }
    }

    fn is_paused(&self) -> bool {
        self.speed_level == 0
    }

    fn get_time(&mut self) -> f32 {
        let mut time = self.anchor_time
            + self.instant.elapsed().as_secs_f32() * self.speed.unwrap_or_default();
        if !self.time_interval.contains(&time) {
            time = time.clamp(self.time_interval.start, self.time_interval.end);
            self.speed = None;
            self.anchor_time = time;
            self.instant = Instant::now();
        }
        time
    }

    fn offset_time(&mut self, delta_time: f32) -> f32 {
        let mut time = self.get_time() + delta_time;
        if !self.time_interval.contains(&time) {
            time = time.clamp(self.time_interval.start, self.time_interval.end);
            self.speed = None;
        }
        self.anchor_time = time;
        self.instant = Instant::now();
        time
    }

    fn set_speed(&mut self, speed: Option<f32>) {
        self.anchor_time = self.get_time();
        self.instant = Instant::now();
        self.speed = speed;
    }

    fn pause(&mut self) {
        self.set_speed(None);
    }

    fn resume(&mut self) {
        self.set_speed(Some(1.0));
    }
}

pub(crate) struct App {
    renderer: Renderer,
    timer: AccumulativeTimer,
    supervisor_data: SupervisorData,
}

impl App {
    pub(crate) fn run(supervisor_data: SupervisorData) -> Result<(), winit::error::EventLoopError> {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut app = Self {
            renderer: None,
            timer: AccumulativeTimer::new(supervisor_data.full_time(), 1.0),
            supervisor_data,
        };
        event_loop.run_app(&mut app)
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.renderer.is_none() {
            self.renderer = Some(pollster::block_on(Renderer::new(
                event_loop
                    .create_window(winit::window::Window::default_attributes().with_inner_size(
                        winit::dpi::PhysicalSize {
                            width: 1600,
                            height: 900,
                        },
                    ))
                    .unwrap(),
            )));
            self.timer.resume();
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
                if !self.timer.is_paused() {
                    self.supervisor_data
                        .present_all(self.timer.get_time(), self.renderer.as_ref().unwrap())
                }
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::MouseWheel {
                delta: winit::event::MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.timer.offset_time(5.0 * (x + y));
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // self.renderer.request_redraw()
        self.renderer.as_ref().unwrap().window.request_redraw();
    }
}
