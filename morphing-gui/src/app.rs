// use std::path::PathBuf;

// use morphing_core::config::Config;
// use morphing_core::scene::read_and_deserialize;
// use morphing_core::scene::LineResult;
// use morphing_core::scene::RedirectedResult;
// use morphing_core::scene::SceneData;
// use morphing_core::timeline::TimelineEntries;

// use super::collection::Collection;
// use super::collection::CollectionItem;
// use super::io::compile_project;
// use super::io::pick_folder;
// use super::io::pick_folders;
// use super::io::pick_save_file;
// use super::logger::LogLevel;
// use super::logger::Logger;
// use super::progress::Progress;
// use super::progress::ProgressMessage;

// #[derive(Debug)]
// struct ScenePrimitive {
//     time: f32,
//     timeline_entries: TimelineEntries,
//     resolution: iced::Size<u32>,
//     fps: f64,
//     background_color: iced::widget::shader::wgpu::Color,
// }

// impl ScenePrimitive {
//     fn new(scene_success_state: &SceneSuccessState) -> Self {
//         let config = &scene_success_state.config;
//         Self {
//             time: scene_success_state.progress.time(),
//             timeline_entries: scene_success_state.timeline_entries.clone(),
//             resolution: config.get_cloned("camera.resolution"),
//             fps: config.get_cloned("camera.fps"),
//             background_color: config.get_cloned("style.background_color"),
//         }
//     }
// }

// impl iced::widget::shader::Primitive for ScenePrimitive {
//     fn prepare(
//         &self,
//         device: &iced::widget::shader::wgpu::Device,
//         queue: &iced::widget::shader::wgpu::Queue,
//         format: iced::widget::shader::wgpu::TextureFormat,
//         storage: &mut iced::widget::shader::Storage,
//         bounds: &iced::Rectangle,
//         viewport: &iced::widget::shader::Viewport,
//     ) {
//         self.timeline_entries
//             .prepare(self.time, device, queue, format, storage, bounds, viewport);
//     }

//     fn render(
//         &self,
//         encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
//         storage: &iced::widget::shader::Storage,
//         target: &iced::widget::shader::wgpu::TextureView,
//         clip_bounds: &iced::Rectangle<u32>,
//     ) {
//         {
//             let mut render_pass =
//                 encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
//                     label: None,
//                     color_attachments: &[Some(
//                         iced::widget::shader::wgpu::RenderPassColorAttachment {
//                             view: target,
//                             resolve_target: None,
//                             ops: iced::widget::shader::wgpu::Operations {
//                                 load: iced::widget::shader::wgpu::LoadOp::Clear(
//                                     self.background_color,
//                                 ),
//                                 store: iced::widget::shader::wgpu::StoreOp::Store,
//                             },
//                         },
//                     )],
//                     depth_stencil_attachment: None,
//                     timestamp_writes: None,
//                     occlusion_query_set: None,
//                 });
//             render_pass.set_scissor_rect(
//                 clip_bounds.x,
//                 clip_bounds.y,
//                 clip_bounds.width,
//                 clip_bounds.height,
//             );
//         }
//         self.timeline_entries
//             .render(self.time, encoder, storage, target, clip_bounds);
//     }
// }

// impl iced::widget::shader::Program<AppMessage> for AppState {
//     type State = ();
//     type Primitive = SceneSuccessState;

//     fn update(
//         &self,
//         _state: &mut Self::State,
//         _event: iced::widget::shader::Event,
//         _bounds: iced::Rectangle,
//         _cursor: iced::mouse::Cursor,
//         shell: &mut iced::advanced::Shell<'_, AppMessage>,
//     ) -> (iced::event::Status, Option<AppMessage>) {
//         if let Some(project_state) = self.projects.get_active() {
//             if let Some(success_project_state) = project_state.project_success_state.as_ref() {
//                 if let Some(scene_state) = success_project_state.scenes.get_active() {
//                     if let Some(success_scene_state) = scene_state.scene_success_state.as_ref() {
//                         if success_scene_state.progress.is_playing() {
//                             shell.request_redraw(iced::window::RedrawRequest::NextFrame);
//                         }
//                     }
//                 }
//             }
//         }
//         (iced::event::Status::Ignored, None)
//     }

//     fn draw(
//         &self,
//         _state: &Self::State,
//         _cursor: iced::mouse::Cursor,
//         _bounds: iced::Rectangle,
//     ) -> Self::Primitive {
//         todo!()
//     }
// }

// enum Message {
//     SceneMessage(SceneMessage),
// }

// enum SceneMessage {
//     CompilationRequest(PathBuf),
//     CompilationComplete,
//     ExecutionRequest(PathBuf),
//     ExecutionComplete(SceneTimelines),
//     PresentationRequest(PathBuf),
//     PresentationComplete(ScenePresentations),
// }

// fn update()

// impl App {
//     pub fn new(// presentation_collection: PresentationCollection,
//         // window_config: WindowConfig,
//         // video_config: VideoConfig,
//         // config: Config,
//     ) -> Self {
//         // env_logger::init();
//         // let event_loop = winit::event_loop::EventLoop::new().unwrap();
//         Self {
//             settings: Settings::default(),
//             modules: Vec::new(),

//             progress: Progress, // renderer: OnceLock::new(),
//                                 // progress: Progress::new(presentation_collection.full_time()),
//                                 // control_pressed: false,
//                                 // presentation_collection: None,
//         }
//         // event_loop.run_app(&mut app)
//     }

//     fn render(&self, time: f32) {
//         self.presentation_collection
//             .present_all(time, self.renderer.get().unwrap())
//     }

//     fn on_redraw_requested(&mut self) {
//         if self.progress.speed_level != 0 {
//             let time = self.progress.get_time();
//             self.render(time);
//         }
//     }

//     fn on_key_down(&mut self, key: winit::keyboard::Key, control_pressed: bool) {
//         match key {
//             winit::keyboard::Key::Named(named_key) => match named_key {
//                 winit::keyboard::NamedKey::ArrowRight if !control_pressed => {
//                     let time = self
//                         .progress
//                         .forward_time(self.window_config.forward_seconds);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::ArrowRight if control_pressed => {
//                     let time = self
//                         .progress
//                         .forward_time(self.window_config.fast_forward_seconds);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::ArrowLeft if !control_pressed => {
//                     let time = self
//                         .progress
//                         .forward_time(-self.window_config.forward_seconds);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::ArrowLeft if control_pressed => {
//                     let time = self
//                         .progress
//                         .forward_time(-self.window_config.fast_forward_seconds);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::ArrowUp => {
//                     let time = self
//                         .progress
//                         .set_speed_level(|speed_level| speed_level.max(0) + 1);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::ArrowDown => {
//                     let time = self
//                         .progress
//                         .set_speed_level(|speed_level| speed_level.min(0) - 1);
//                     self.render(time);
//                 }
//                 winit::keyboard::NamedKey::Space => {
//                     let time = self
//                         .progress
//                         .set_speed_level(|speed_level| if speed_level != 0 { 0 } else { 1 });
//                     self.render(time);
//                 }
//                 _ => {}
//             },
//             winit::keyboard::Key::Character(ch) => match ch.as_str() {
//                 "s" if control_pressed => {
//                     let time = self.progress.set_speed_level(|_| 0);
//                     self.render(time);
//                     if let Some(save_file) = rfd::AsyncFileDialog::new()
//                         .add_filter("MP4", &["mp4"])
//                         .add_filter("PNG", &["png"])
//                         .save_file()
//                         .block_on()
//                     {
//                         let path: PathBuf = save_file.into();
//                         match path.extension().map(OsStr::to_str).flatten() {
//                             Some("mp4") => self.save_video(path),
//                             Some("png") => self.save_image(path),
//                             _ => panic!("Unsupported output file extension: {path:?}"),
//                         }
//                     }
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//     }

//     fn save_video(&self, path: PathBuf) {
//         let mut ffmpeg = essi_ffmpeg::FFmpeg::new()
//             .stderr(Stdio::inherit())
//             .input_with_file("-".into())
//             .done()
//             .output_as_file(path)
//             .done()
//             .start()
//             .unwrap();

//         let texture = self.renderer.get().unwrap().create_texture();
//         let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

//         let full_time = self.progress.time_interval.end;
//         let fps = self.video_config.fps;
//         // (0..=(full_time / fps).ceil() as u32).for_each(|i| i as f32 * fps)
//         // ffmpeg.stdin()
//     }

//     fn save_image(&self, path: PathBuf) {
//         todo!()
//     }
// }

// impl winit::application::ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
//         self.renderer.get_or_init(|| {
//             let window = event_loop
//                 .create_window(
//                     winit::window::Window::default_attributes()
//                         .with_inner_size::<winit::dpi::PhysicalSize<u32>>(
//                             winit::dpi::PhysicalSize::from(self.window_config.size),
//                         ),
//                 )
//                 .unwrap();
//             let renderer = Renderer::new(window).unwrap();
//             self.progress.set_base_speed(self.window_config.base_speed);
//             self.progress.set_speed_level(|_| 1);
//             renderer
//         });
//     }

//     fn window_event(
//         &mut self,
//         event_loop: &winit::event_loop::ActiveEventLoop,
//         _window_id: winit::window::WindowId,
//         event: winit::event::WindowEvent,
//     ) {
//         match event {
//             winit::event::WindowEvent::RedrawRequested => {
//                 self.on_redraw_requested();
//             }
//             winit::event::WindowEvent::CloseRequested => event_loop.exit(),
//             winit::event::WindowEvent::ModifiersChanged(modifiers) => {
//                 self.control_pressed = modifiers.state().control_key();
//             }
//             winit::event::WindowEvent::KeyboardInput {
//                 event:
//                     winit::event::KeyEvent {
//                         logical_key,
//                         state: winit::event::ElementState::Pressed,
//                         ..
//                     },
//                 ..
//             } => {
//                 self.on_key_down(logical_key, self.control_pressed);
//             }
//             _ => {}
//         };
//     }

//     fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
//         self.renderer.get().unwrap().request_redraw();
//     }
// }
