use std::path::PathBuf;
use std::sync::Arc;

use morphing::timelines::timeline::TimelineEntries;
use morphing::toplevel::scene::SceneData;
use morphing::toplevel::scene::SceneResult;
use morphing::toplevel::settings::SceneSettings;
use morphing::toplevel::settings::VideoSettings;

use super::collection::Collection;
use super::io::compile_project;
use super::io::open_project;
use super::logger::Logger;
use super::progress::Progress;
use super::progress::ProgressMessage;

// #[derive(Clone, Debug)]
// struct PlayerSettings {
//     play_pause_key: iced::keyboard::Key,
//     fast_forward_key: iced::keyboard::Key,
//     fast_backward: iced::keyboard::Key,
//     fast_skip_seconds: f32,
// }

// impl Default for PlayerSettings {
//     fn default() -> Self {
//         Self {
//             play_pause_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
//             fast_forward_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
//             fast_backward: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
//             fast_skip_seconds: 5.0,
//         }
//     }
// }

#[derive(Debug, Default)]
pub(crate) struct AppState {
    projects: Collection<ProjectState>,
    scene_settings: Arc<SceneSettings>,
}

#[derive(Debug)]
struct ProjectState {
    path: PathBuf,
    watching: bool, // TODO
    project_success_state: Option<ProjectSuccessState>,
    logger: Logger,
}

#[derive(Debug)]
struct ProjectSuccessState {
    scenes: Collection<SceneState>,
}

#[derive(Debug)]
struct SceneState {
    name: String,
    scene_success_state: Option<SceneSuccessState>,
    logger: Logger,
}

#[derive(Debug)]
struct SceneSuccessState {
    progress: Progress,
    timeline_entries: TimelineEntries,
    video_settings: VideoSettings,
}

#[derive(Debug)]
pub enum AppMessage {
    Open(PathBuf),
    OpenResult(anyhow::Result<PathBuf>),
    Close(usize),
    Activate(Option<usize>),
    ProjectState(PathBuf, ProjectStateMessage),
}

#[derive(Debug)]
pub enum ProjectStateMessage {
    Compile(Arc<SceneSettings>),
    CompileResult(anyhow::Result<Vec<(String, SceneData)>>),
    SetWatching(bool),
    ProjectSuccessState(ProjectSuccessStateMessage),
}

#[derive(Debug)]
pub enum ProjectSuccessStateMessage {
    Activate(Option<usize>),
    SceneState(String, SceneStateMessage),
}

#[derive(Debug)]
pub enum SceneStateMessage {
    Load(SceneData),
    SceneSuccessState(SceneSuccessStateMessage),
}

#[derive(Debug)]
pub enum SceneSuccessStateMessage {
    SetTimelineEntries(f32, TimelineEntries),
    SetVideoSettings(VideoSettings),
    SaveVideo,
    SaveImage(f32),
    Progress(ProgressMessage),
}

impl AppState {
    // fn active_project_state(&self) -> Option<&ProjectState> {
    //     self.projects.get_active()
    // }

    // fn active_project_state_mut(&mut self) -> Option<&mut ProjectState> {
    //     self.projects.get_active_mut()
    // }

    // fn active_project_state(&self) -> Option<&ProjectState> {
    //     self.active_project
    //         .as_ref()
    //         .map(|active_project| self.projects.get(active_project))
    //         .flatten()
    // }

    // fn active_project_state_mut(&mut self) -> Option<&mut ProjectState> {
    //     self.active_project
    //         .as_mut()
    //         .map(|active_project| self.projects.get_mut(active_project))
    //         .flatten()
    // }

    // fn active_scene_state(&self) -> Option<&SceneState> {
    //     self.active_project_state()
    //         .map(|project_state| project_state.active_scene_state())
    //         .flatten()
    // }

    // fn active_scene_state_mut(&mut self) -> Option<&mut SceneState> {
    //     self.active_project_state_mut()
    //         .map(|project_state| project_state.active_scene_state_mut())
    //         .flatten()
    // }

    pub(crate) fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Open(path) => {
                iced::Task::perform(open_project(path), AppMessage::OpenResult)
            }
            AppMessage::OpenResult(result) => {
                match result {
                    Ok(path) => iced::Task::done(AppMessage::ProjectState(
                        path,
                        ProjectStateMessage::Compile(self.scene_settings.clone()),
                    )),
                    Err(error) => {
                        todo!(); // TODO: error window
                        iced::Task::none()
                    }
                }
            }
            AppMessage::Close(index) => {
                self.projects.remove(index);
                iced::Task::none()
            }
            AppMessage::Activate(index) => {
                self.projects.set_active_index(index);
                iced::Task::none()
            }
            AppMessage::ProjectState(path, message) => self
                .projects
                .find_or_insert_with(
                    |project_state| project_state.path == path,
                    || ProjectState {
                        path: path.clone(),
                        watching: false,
                        project_success_state: None,
                        logger: Logger::default(),
                    },
                )
                .update(message)
                .map(move |message| AppMessage::ProjectState(path.clone(), message)),
        }
    }

    pub(crate) fn view(&self) -> iced::Element<AppMessage> {
        iced::widget::Shader::new(self).into()
    }
}

impl ProjectState {
    fn update(&mut self, message: ProjectStateMessage) -> iced::Task<ProjectStateMessage> {
        match message {
            ProjectStateMessage::Compile(scene_settings) => {
                self.logger.log(log::Level::Trace, "Compilation starts");
                iced::Task::perform(
                    compile_project(self.path.clone(), scene_settings),
                    move |result| ProjectStateMessage::CompileResult(result),
                )
            }
            ProjectStateMessage::CompileResult(result) => match result {
                Ok(scenes_data) => {
                    self.logger.log(log::Level::Trace, "Compilation ends");
                    iced::Task::batch(scenes_data.into_iter().map(|(name, scene_data)| {
                        iced::Task::done(ProjectStateMessage::ProjectSuccessState(
                            ProjectSuccessStateMessage::SceneState(
                                name,
                                SceneStateMessage::Load(scene_data),
                            ),
                        ))
                    }))
                }
                Err(error) => {
                    self.logger.log(log::Level::Error, error);
                    self.logger.log(log::Level::Trace, "Compilation fails");
                    iced::Task::none()
                }
            },
            ProjectStateMessage::SetWatching(watching) => {
                self.watching = watching;
                todo!();
                iced::Task::none()
            }
            ProjectStateMessage::ProjectSuccessState(message) => self
                .project_success_state
                .get_or_insert_with(|| ProjectSuccessState {
                    scenes: Collection::default(),
                })
                .update(message)
                .map(ProjectStateMessage::ProjectSuccessState),
        }
    }
}

impl ProjectSuccessState {
    fn update(
        &mut self,
        message: ProjectSuccessStateMessage,
    ) -> iced::Task<ProjectSuccessStateMessage> {
        match message {
            ProjectSuccessStateMessage::Activate(index) => {
                self.scenes.set_active_index(index);
                iced::Task::none()
            }
            ProjectSuccessStateMessage::SceneState(name, message) => self
                .scenes
                .find_or_insert_with(
                    |scene_state| scene_state.name == name,
                    || SceneState {
                        name: name.clone(),
                        scene_success_state: None,
                        logger: Logger::default(),
                    },
                )
                .update(message)
                .map(move |message| ProjectSuccessStateMessage::SceneState(name.clone(), message)),
        }
    }
}

impl SceneState {
    fn update(&mut self, message: SceneStateMessage) -> iced::Task<SceneStateMessage> {
        match message {
            SceneStateMessage::Load(scene_data) => match scene_data.result {
                SceneResult::Success {
                    time,
                    timeline_entries,
                    video_settings,
                } => {
                    self.logger.log(log::Level::Trace, "Loading starts");
                    for line in scene_data.stdout_lines {
                        self.logger.log(log::Level::Info, line);
                    }
                    for line in scene_data.stderr_lines {
                        self.logger.log(log::Level::Error, line);
                    }
                    self.logger.log(log::Level::Trace, "Loading ends");
                    iced::Task::done(SceneStateMessage::SceneSuccessState(
                        SceneSuccessStateMessage::SetTimelineEntries(time, timeline_entries),
                    ))
                    .chain(iced::Task::done(
                        SceneStateMessage::SceneSuccessState(
                            SceneSuccessStateMessage::SetVideoSettings(video_settings),
                        ),
                    ))
                }
                SceneResult::Error => {
                    self.logger.log(log::Level::Trace, "Loading starts");
                    for line in scene_data.stdout_lines {
                        self.logger.log(log::Level::Info, line);
                    }
                    for line in scene_data.stderr_lines {
                        self.logger.log(log::Level::Error, line);
                    }
                    self.logger.log(log::Level::Trace, "Loading fails");
                    iced::Task::none()
                }
                SceneResult::Skipped => {
                    self.logger.log(log::Level::Trace, "Loading skipped");
                    iced::Task::none()
                }
            },
            SceneStateMessage::SceneSuccessState(message) => self
                .scene_success_state
                .get_or_insert_with(|| SceneSuccessState {
                    progress: Progress::new(0.0),
                    timeline_entries: TimelineEntries::default(),
                    video_settings: VideoSettings::default(),
                })
                .update(message)
                .map(SceneStateMessage::SceneSuccessState),
        }
    }
}

impl SceneSuccessState {
    fn update(
        &mut self,
        message: SceneSuccessStateMessage,
    ) -> iced::Task<SceneSuccessStateMessage> {
        match message {
            SceneSuccessStateMessage::SetTimelineEntries(time, timeline_entries) => {
                self.progress = Progress::new(time);
                self.timeline_entries = timeline_entries;
                iced::Task::none()
            }
            SceneSuccessStateMessage::SetVideoSettings(video_settings) => {
                self.video_settings = video_settings;
                iced::Task::none()
            }
            SceneSuccessStateMessage::SaveVideo => {
                todo!();
                iced::Task::none()
            }
            SceneSuccessStateMessage::SaveImage(time) => {
                todo!();
                iced::Task::none()
            }
            SceneSuccessStateMessage::Progress(message) => self
                .progress
                .update(message)
                .map(SceneSuccessStateMessage::Progress),
        }
    }
}

// #[derive(Debug)]
// pub struct Primitive(Option<SceneStatus>); // TODO: remove option

impl iced::widget::shader::Primitive for SceneSuccessState {
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        self.timeline_entries.prepare(
            self.progress.get_time(),
            device,
            queue,
            format,
            storage,
            bounds,
            viewport,
        );
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        {
            let background_color = iced::widget::shader::wgpu::Color {
                r: self.video_settings.background_color.red as f64,
                g: self.video_settings.background_color.green as f64,
                b: self.video_settings.background_color.blue as f64,
                a: self.video_settings.background_color.alpha as f64,
            };
            let mut render_pass =
                encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        iced::widget::shader::wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops: iced::widget::shader::wgpu::Operations {
                                load: iced::widget::shader::wgpu::LoadOp::Clear(background_color),
                                store: iced::widget::shader::wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            render_pass.set_scissor_rect(
                clip_bounds.x,
                clip_bounds.y,
                clip_bounds.width,
                clip_bounds.height,
            );
        }
        self.timeline_entries.render(
            self.progress.get_time(),
            encoder,
            storage,
            target,
            clip_bounds,
        );
    }
}

impl iced::widget::shader::Program<AppMessage> for AppState {
    type State = ();
    type Primitive = SceneSuccessState;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::shader::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, AppMessage>,
    ) -> (iced::event::Status, Option<AppMessage>) {
        if let Some(project_state) = self.projects.get_active() {
            if let Some(success_project_state) = project_state.project_success_state.as_ref() {
                if let Some(scene_state) = success_project_state.scenes.get_active() {
                    if let Some(success_scene_state) = scene_state.scene_success_state.as_ref() {
                        if success_scene_state.progress.is_playing() {
                            shell.request_redraw(iced::window::RedrawRequest::NextFrame);
                        }
                    }
                }
            }
        }
        (iced::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        todo!()
    }
}

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
