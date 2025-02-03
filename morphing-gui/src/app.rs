use std::ops::RangeInclusive;
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
use super::logger::LoggingCategory;
use super::progress::Progress;
use super::progress::ProgressMessage;

const PROGRESS_SPEED_LEVEL_RANGE: RangeInclusive<i32> = -5..=5;
const PLAY_PAUSE_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::Space);
const FAST_FORWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight);
const FAST_BACKWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft);
const FAST_SKIP_SECONDS: f32 = 5.0;

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
    // projects: Vec<ProjectState>,
    // active_project_path: Option<PathBuf>,
    scene_settings: Arc<SceneSettings>,
}

#[derive(Debug)]
struct ProjectState {
    path: PathBuf,
    watching: bool, // TODO
    project_status: ProjectStatus,
    logger: Logger,
}

#[derive(Debug)]
enum ProjectStatus {
    Success(ProjectSuccessStatus),
    Error,
    Compiling,
}

#[derive(Debug)]
struct ProjectSuccessStatus {
    scenes: Collection<SceneState>,
}

#[derive(Debug)]
struct SceneState {
    name: String,
    scene_status: SceneStatus,
    logger: Logger,
}

#[derive(Debug)]
enum SceneStatus {
    Success(SceneSuccessStatus),
    Error,
    Skipped,
}

#[derive(Debug)]
struct SceneSuccessStatus {
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
    CompileResult(anyhow::Result<Vec<SceneData>>),
    ProjectSuccessStatus(ProjectSuccessStatusMessage),
}

#[derive(Debug)]
pub enum ProjectSuccessStatusMessage {
    Activate(Option<usize>),
    SceneState(String, SceneStateMessage),
}

#[derive(Debug)]
pub enum SceneStateMessage {
    SceneSuccessStatus(SceneSuccessStatusMessage),
}

#[derive(Debug)]
pub enum SceneSuccessStatusMessage {
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

    //     impl ProjectState {
    //     fn active_scene_state(&self) -> Option<&SceneState> {
    //         if let ProjectStatus::Success {
    //             scenes,
    //             active_scene,
    //         } = &self.project_status
    //         {
    //             active_scene
    //                 .as_ref()
    //                 .map(|active_project| scenes.get(active_project))
    //                 .flatten()
    //         } else {
    //             None
    //         }
    //     }

    //     fn active_scene_state_mut(&mut self) -> Option<&mut SceneState> {
    //         if let ProjectStatus::Success {
    //             scenes,
    //             active_scene,
    //         } = &mut self.project_status
    //         {
    //             active_scene
    //                 .as_mut()
    //                 .map(|active_project| scenes.get_mut(active_project))
    //                 .flatten()
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub(crate) fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Open(path) => {
                iced::Task::perform(open_project(path), AppMessage::OpenResult)
            }
            AppMessage::OpenResult(result) => {
                match result {
                    Err(error) => {
                        todo!(); // TODO: error window
                        iced::Task::none()
                    }
                    Ok(path) => {
                        self.projects.insert_with(
                            |project_state| project_state.path == path,
                            || ProjectState {
                                path: path.clone(),
                                watching: false,
                                project_status: ProjectStatus::Compiling,
                                logger: Logger::default(),
                            },
                        );
                        iced::Task::done(AppMessage::ProjectState(
                            path,
                            ProjectStateMessage::Compile(self.scene_settings.clone()),
                        ))
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
            AppMessage::ProjectState(path, message) => {
                if let Some(project_state) = self
                    .projects
                    .find(|project_state| &project_state.path == &path)
                {
                    project_state
                        .update(message)
                        .map(move |message| AppMessage::ProjectState(path.clone(), message))
                } else {
                    iced::Task::none()
                }
            } // AppMessage::ProjectCompile(path) => {
              //     if let Some(project_state) = self.find(|project_state| project_state.path == path) {
              //     } else {
              //         iced::Task::none()
              //     }
              // }
              // AppMessage::ProjectCompileResult(path, compile_result) => {
              //     if let Some(project_state) = self.find(|project_state| project_state.path == path) {
              //     }
              //     iced::Task::none()
              // }
              // AppMessage::SceneActivate(index) => {
              //     if let Some(project_state) = self.projects.get_active_mut() {
              //         if let ProjectStatus::Success { scenes } = &mut project_state.project_status {
              //             scenes.set_active_index(index);
              //         }
              //     }
              //     iced::Task::none()
              // }
              // AppMessage::ProgressSetTime(time) => {
              //     if let Some(project_state) = self.projects.get_active_mut() {
              //         if let ProjectStatus::Success { scenes } = &mut project_state.project_status {
              //             if let Some(scene_state) = scenes.get_active_mut() {
              //                 if let SceneStatus::Success { progress, .. } =
              //                     &mut scene_state.scene_status
              //                 {
              //                     progress.set_time(time);
              //                 }
              //             }
              //         }
              //     }
              //     iced::Task::none()
              // }
              // AppMessage::ProgressSetSpeedLevel(speed_level) => {
              //     if let Some(scene_state) = self.active_scene_state_mut() {
              //         if let SceneStatus::Success { progress, .. } = &mut scene_state.scene_status {
              //             progress.set_speed_level(speed_level);
              //         }
              //     }
              //     iced::Task::none()
              // }
              // AppMessage::ProgressSetPlayDirection(play_direction) => {
              //     if let Some(scene_state) = self.active_scene_state_mut() {
              //         if let SceneStatus::Success { progress, .. } = &mut scene_state.scene_status {
              //             progress.set_play_direction(play_direction);
              //         }
              //     }
              //     iced::Task::none()
              // }
              // AppMessage::ProgressSetPlaying(playing) => {
              //     if let Some(scene_state) = self.active_scene_state_mut() {
              //         if let SceneStatus::Success { progress, .. } = &mut scene_state.scene_status {
              //             progress.set_playing(playing);
              //         }
              //     }
              //     iced::Task::none()
              // }
        }
    }

    pub(crate) fn view(&self) -> iced::Element<AppMessage> {
        iced::widget::Shader::new(self).into()
    }

    // fn prepare(
    //     timeline_entries: Arc<TimelineEntries>,
    //     device: Arc<wgpu::Device>,
    // ) -> anyhow::Result<PresentationEntries> {
    //     timeline_entries.prepare(&device)
    // }
}

impl ProjectState {
    fn update(&mut self, message: ProjectStateMessage) -> iced::Task<ProjectStateMessage> {
        match message {
            ProjectStateMessage::Compile(scene_settings) => {
                self.project_status = ProjectStatus::Compiling;
                self.logger
                    .log_line(LoggingCategory::Stdin, String::from("Compiling"));
                iced::Task::perform(
                    compile_project(self.path.clone(), scene_settings),
                    move |result| ProjectStateMessage::CompileResult(result),
                )
            }
            ProjectStateMessage::CompileResult(result) => {
                self.project_status = match result {
                    Err(error) => {
                        self.logger
                            .log_line(LoggingCategory::Stderr, error.to_string());
                        ProjectStatus::Error
                    }
                    Ok(scenes) => ProjectStatus::Success(ProjectSuccessStatus {
                        scenes: scenes
                            .into_iter()
                            .map(|scene_data| {
                                let scene_status = match scene_data.result {
                                    SceneResult::Success {
                                        time,
                                        timeline_entries,
                                        video_settings,
                                    } => SceneStatus::Success(SceneSuccessStatus {
                                        progress: Progress::new(time),
                                        timeline_entries,
                                        video_settings,
                                    }),
                                    SceneResult::Error => SceneStatus::Error,
                                    SceneResult::Skipped => SceneStatus::Skipped,
                                };
                                let mut logger = Logger::default();
                                logger.log_lines(LoggingCategory::Stdout, scene_data.stdout_lines);
                                logger.log_lines(LoggingCategory::Stderr, scene_data.stderr_lines);
                                SceneState {
                                    name: scene_data.name,
                                    scene_status,
                                    logger,
                                }
                            })
                            .collect(),
                    }),
                };
                iced::Task::none()
            }
            ProjectStateMessage::ProjectSuccessStatus(message) => {
                if let ProjectStatus::Success(project_success_status) = &mut self.project_status {
                    project_success_status
                        .update(message)
                        .map(ProjectStateMessage::ProjectSuccessStatus)
                } else {
                    iced::Task::none()
                }
            }
        }
    }
}

impl ProjectSuccessStatus {
    fn update(
        &mut self,
        message: ProjectSuccessStatusMessage,
    ) -> iced::Task<ProjectSuccessStatusMessage> {
        match message {
            ProjectSuccessStatusMessage::Activate(index) => {
                self.scenes.set_active_index(index);
                iced::Task::none()
            }
            ProjectSuccessStatusMessage::SceneState(name, message) => {
                if let Some(scene_state) = self.scenes.find(|scene_state| scene_state.name == name)
                {
                    scene_state.update(message).map(move |message| {
                        ProjectSuccessStatusMessage::SceneState(name.clone(), message)
                    })
                } else {
                    iced::Task::none()
                }
            }
        }
    }
}

impl SceneState {
    fn update(&mut self, message: SceneStateMessage) -> iced::Task<SceneStateMessage> {
        match message {
            SceneStateMessage::SceneSuccessStatus(message) => {
                if let SceneStatus::Success(scene_success_status) = &mut self.scene_status {
                    scene_success_status
                        .update(message)
                        .map(SceneStateMessage::SceneSuccessStatus)
                } else {
                    iced::Task::none()
                }
            }
        }
    }
}

impl SceneSuccessStatus {
    fn update(
        &mut self,
        message: SceneSuccessStatusMessage,
    ) -> iced::Task<SceneSuccessStatusMessage> {
        match message {
            SceneSuccessStatusMessage::SaveVideo => {
                todo!();
                iced::Task::none()
            }
            SceneSuccessStatusMessage::SaveImage(time) => {
                todo!();
                iced::Task::none()
            }
            SceneSuccessStatusMessage::Progress(message) => self
                .progress
                .update(message)
                .map(SceneSuccessStatusMessage::Progress),
        }
    }
}

// #[derive(Debug)]
// pub struct SceneState {

//     timeline_collection: Option<SceneTimelineCollection>,
//     // video_settings: VideoSettings,
//     // duration: f32,
//     // status: SceneStatus,
//     // timeline_entries: TimelineEntries,
// }

// enum SceneStatus {
//     BeforePrepare,
//     OnPrepare,
//     AfterPrepare(PresentationEntries),
//     PrepareError(anyhow::Error),
//     PresentError(anyhow::Error),
// }

// impl Default for Progress {
//     fn default() -> Self {
//         Self::new(0.0, 1.0)
//     }
// }

// #[derive(Debug)]
// pub struct Primitive(Option<SceneStatus>); // TODO: remove option

impl iced::widget::shader::Primitive for SceneSuccessStatus {
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
    type Primitive = SceneSuccessStatus;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::shader::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, AppMessage>,
    ) -> (iced::event::Status, Option<AppMessage>) {
        if let Some(project_state) = self.projects.get_active() {
            if let ProjectStatus::Success(success_project_status) = &project_state.project_status {
                if let Some(scene_state) = success_project_status.scenes.get_active() {
                    if let SceneStatus::Success(success_scene_status) = &scene_state.scene_status {
                        if success_scene_status.progress.is_playing() {
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
