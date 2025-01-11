use std::time::Duration;
use std::time::Instant;

use super::renderer::Renderer;
use super::scene::ArchivedPresentations;

struct AccumulativeTimer {
    anchor_time: Duration,
    instant: Instant,
    speed: Option<f32>,
}

impl AccumulativeTimer {
    fn new() -> Self {
        Self {
            anchor_time: Duration::ZERO,
            instant: Instant::now(),
            speed: None,
        }
    }

    fn get_time(&self) -> Duration {
        self.anchor_time
            + self
                .instant
                .elapsed()
                .mul_f32(self.speed.unwrap_or_default())
    }

    fn is_paused(&self) -> bool {
        self.speed.is_none()
    }

    fn set_time(&mut self, time: Duration) {
        self.anchor_time = time;
        self.instant = Instant::now();
    }

    fn offset_time(&mut self, delta_time: Duration) {
        self.set_time(self.anchor_time + delta_time);
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

struct App {
    renderer: Option<Renderer>,
    timer: AccumulativeTimer,
    presentations: ArchivedPresentations,
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.renderer.get_or_insert_with(|| {
            pollster::block_on(Renderer::new(
                event_loop
                    .create_window(winit::window::Window::default_attributes().with_inner_size(
                        winit::dpi::PhysicalSize {
                            width: 1600,
                            height: 900,
                        },
                    ))
                    .unwrap(),
            ))
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
                self.presentations.present_all(
                    self.timer.get_time().as_secs_f32(),
                    self.renderer.as_ref().unwrap(),
                );
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        };
    }
}
