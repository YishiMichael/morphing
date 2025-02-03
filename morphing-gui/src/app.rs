
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use morphing::timelines::timeline::TimelineEntries;
use morphing::toplevel::scene::SceneData;
use morphing::toplevel::scene::SceneResult;
use morphing::toplevel::settings::SceneSettings;
use morphing::toplevel::settings::VideoSettings;

const PROGRESS_SPEED_LEVEL_RANGE: RangeInclusive<i32> = -5..=5;
const PLAY_PAUSE_KEY: iced::keyboard::Key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Space);
const FAST_FORWARD_KEY: iced::keyboard::Key = iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight);
const FAST_BACKWARD_KEY: iced::keyboard::Key = iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft);
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
    projects: indexmap::IndexMap<PathBuf, ProjectState>,
    active_path: Option<PathBuf>,
    scene_settings: Arc<SceneSettings>,
    // player_settings: PlayerSettings,
    // progress: Progress,
    // active_scene: Option<SceneData>,
}

#[derive(Debug)]
struct ProjectState {
    project_status: ProjectStatus,
    logger: Logger,
}

#[derive(Debug)]
enum ProjectStatus {
    BeforeCompile,
    OnCompile,
    AfterCompile {
        scenes: indexmap::IndexMap<String, SceneState>,
        active_name: Option<String>,
    },
    CompileError(anyhow::Error),
}

#[derive(Debug)]
struct SceneState {
    scene_status: SceneStatus,
    logger: Logger,
}

#[derive(Clone, Debug)]
enum SceneStatus {
    Success {
        progress: Progress,
        timeline_entries: TimelineEntries,
        video_settings: VideoSettings,
    },
    Error,
    Skipped,
}

#[derive(Clone, Debug)]
struct Progress {
    time_interval: Range<f32>,
    // anchor_time: f32,
    // instant: Instant,
    time: f32,
    // base_speed: f32,
    progress_speed_level: i32,
    progress_direction: ProgressDirection,
    paused: bool,
}

#[derive(Clone, Debug)]
enum ProgressDirection {
    Forward,
    Backward,
}

#[derive(Debug, Default)]
struct Logger(Vec<(LoggingCategory, String)>);

#[derive(Debug)]
enum LoggingCategory {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub enum AppMessage {
    AddProject(PathBuf),
    RemoveProject(PathBuf),
    CompileProject(PathBuf),
    CompileProjectResult(PathBuf, anyhow::Result<Vec<SceneData>>),
    // Prepare(PathBuf, String),
    // PrepareResult(PathBuf, String, anyhow::Result<PresentationEntries>),
    // PresentError(PathBuf, String, anyhow::Error),
}

impl AppState {
    fn active_project_state(&self) -> Option<&ProjectState> {
        self.active_path.as_ref().map(|active_path| self.projects.get(active_path)).flatten()
    }

    fn active_project_state_mut(&mut self) -> Option<&mut ProjectState> {
        self.active_path.as_mut().map(|active_path| self.projects.get_mut(active_path)).flatten()
    }

    fn active_scene_state(&self) -> Option<&SceneState> {
        self.active_project_state().map(|project_state| project_state.active_scene_state()).flatten()
    }

    fn active_scene_state_mut(&mut self) -> Option<&mut SceneState> {
        self.active_project_state_mut().map(|project_state| project_state.active_scene_state_mut()).flatten()
    }

    async fn compile(
        path: PathBuf,
        scene_settings: Arc<SceneSettings>,
    ) -> anyhow::Result<Vec<SceneData>> {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--quiet")
            .current_dir(path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        writeln!(
            child.stdin.take().unwrap(),
            "{}",
            ron::ser::to_string(&*scene_settings)?
        )?;
        if !child.wait()?.success() {
            let mut stderr = BufReader::new(child.stderr.take().unwrap());
            let mut buf = String::new();
            stderr.read_to_string(&mut buf)?;
            Err(anyhow::Error::msg(buf))?;
        }
        let mut scenes = Vec::new();
        for line in BufReader::new(child.stdout.take().unwrap()).lines() {
            let scene_data: SceneData = ron::de::from_str(&line?)?;
            scenes.push(scene_data);
        }
        Ok(scenes)
        // let mut stdout = BufReader::new(child.stdout.take().unwrap());
        // let mut buf = String::new();
        // let mut scenes = Vec::new();

        // while stdout.read_line(&mut buf)? != 0 {
        //     let (name, scene_data): (String, SceneData) = ron::de::from_str(&buf)?;
        //     scenes.insert(name, scene_data);
        //     buf.clear();
        // }
        // Ok(scenes)
    }

    pub(crate) fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::AddProject(path) => {
                let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
                let path = metadata
                    .packages
                    .iter()
                    .map(|package| package.manifest_path.parent().unwrap())
                    .find(|crate_path| path.starts_with(crate_path))
                    .unwrap()
                    .to_path_buf()
                    .into_std_path_buf();
                self.projects.entry(path).or_insert(ProjectState {
                    project_status: ProjectStatus::BeforeCompile,
                    logger: Logger::default(),
                });
                iced::Task::none()
            }
            AppMessage::RemoveProject(path) => {
                self.projects.shift_remove(&path);
                iced::Task::none()
            }
            AppMessage::CompileProject(path) => {
                if let Some(project_state) = self.projects.get_mut(&path) {
                    *&mut project_state.project_status = ProjectStatus::OnCompile;
                    iced::Task::perform(
                        Self::compile(path.clone(), self.scene_settings.clone()),
                        move |result| AppMessage::CompileProjectResult(path.clone(), result),
                    )
                } else {
                    iced::Task::none()
                }
            }
            AppMessage::CompileProjectResult(path, compile_result) => {
                if let Some(project_state) = self.projects.get_mut(&path) {
                    *&mut project_state.project_status = match compile_result {
                        Err(error) => ProjectStatus::CompileError(error),
                        Ok(scenes) => ProjectStatus::AfterCompile {
                            scenes: scenes
                                .into_iter()
                                .map(|scene_data| {
                                    (
                                        scene_data.name,
                                        SceneState {
                                            scene_status: match scene_data.result {
                                                SceneResult::Success {
                                                    time,
                                                    timeline_entries,
                                                    video_settings,
                                                } => SceneStatus::Success {
                                                    progress: Progress::new(time),
                                                    timeline_entries,
                                                    video_settings,
                                                },
                                                SceneResult::Error => SceneStatus::Error,
                                                SceneResult::Skipped => SceneStatus::Skipped,
                                            },
                                            logger: Logger(
                                                scene_data
                                                    .stdout_lines
                                                    .into_iter()
                                                    .map(|line| (LoggingCategory::Stdout, line))
                                                    .chain(scene_data.stderr_lines.into_iter().map(
                                                        |line| (LoggingCategory::Stderr, line),
                                                    ))
                                                    .collect(),
                                            ),
                                        },
                                    )
                                })
                                .collect(),
                            active_name: None,
                        },
                    };
                }
                iced::Task::none()
            }
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
    fn active_scene_state(&self) -> Option<&SceneState> {
        if let ProjectStatus::AfterCompile { scenes, active_name } = &self.project_status {
            active_name.as_ref().map(|active_path| scenes.get(active_path)).flatten()
        } else {
            None
        }
    }

    fn active_scene_state_mut(&mut self) -> Option<&mut SceneState> {
        if let ProjectStatus::AfterCompile { scenes, active_name } = &mut self.project_status {
            active_name.as_mut().map(|active_path| scenes.get_mut(active_path)).flatten()
        } else {
            None
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

impl Progress {
    fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            time: 0.0,
            // instant: Instant::now(),
            // base_speed,
            progress_speed_level: 0,
            progress_direction: ProgressDirection::Forward,
            paused: true,
        }
    }

    // fn progress_speed(&self) -> ProgressSpeed {
    //     self.progress_speed
    // }

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

    fn advance_time(&mut self, app_delta_time: f32) -> f32 {
        if !self.paused {
            self.time += app_delta_time * match self.progress_direction {
                ProgressDirection::Forward => 1.0,
                ProgressDirection::Backward => -1.0,
            } * 2.0f32.powi(self.progress_speed_level);
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

    fn set_progress_speed_level(&mut self, progress_speed_level: i32) {
        self.progress_speed_level = progress_speed_level;
    }

    fn play_or_pause(&mut self) {
        self.paused = !self.paused;
    }
}

// impl Default for Progress {
//     fn default() -> Self {
//         Self::new(0.0, 1.0)
//     }
// }

#[derive(Debug)]
pub struct Primitive(Option<SceneStatus>);

impl iced::widget::shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        if let Some(SceneStatus::Success { progress, timeline_entries, .. }) = self.0.as_ref() {
            timeline_entries.prepare(progress.time, device, queue, format, storage, bounds, viewport);
        }
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        if let Some(SceneStatus::Success { progress, timeline_entries, video_settings }) = self.0.as_ref() {
            {
                let background_color = iced::widget::shader::wgpu::Color {
                    r: video_settings.background_color.red as f64,
                    g: video_settings.background_color.green as f64,
                    b: video_settings.background_color.blue as f64,
                    a: video_settings.background_color.alpha as f64,
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
            timeline_entries.render(progress.time, encoder, storage, target, clip_bounds);
        }
    }
}

impl iced::widget::shader::Program<AppMessage> for AppState {
    type State = ();
    type Primitive = Primitive;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::shader::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, AppMessage>,
    ) -> (iced::event::Status, Option<AppMessage>) {
        if self.active_scene_state().is_some_and(|scene_state| {
            if let SceneStatus::Success { progress, .. } = &scene_state.scene_status {
                !progress.paused()
            } else {
                false
            }
        }) {
            shell.request_redraw(iced::window::RedrawRequest::NextFrame);
        }
        (iced::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Primitive(self.active_scene_state().map(|scene_state| scene_state.scene_status.clone()))
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
