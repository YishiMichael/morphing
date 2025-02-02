use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::ops::Range;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use morphing::timelines::timeline::TimelineEntries;
use morphing::toplevel::scene::SceneData;
use morphing::toplevel::settings::SceneSettings;

#[derive(Clone, Debug)]
struct PlayerSettings {
    play_pause_key: iced::keyboard::Key,
    fast_forward_key: iced::keyboard::Key,
    fast_backward: iced::keyboard::Key,
    fast_skip_seconds: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            play_pause_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
            fast_forward_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
            fast_backward: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
            fast_skip_seconds: 5.0,
        }
    }
}

#[derive(Default)]
struct ProjectManager {
    projects: indexmap::IndexMap<PathBuf, ProjectData>,
    active: Option<PathBuf>,
}

impl ProjectManager {
    fn update(
        &mut self,
        message: ProjectMessage,
        scene_settings: Arc<SceneSettings>,
    ) -> iced::Task<ProjectMessage> {
        match message {
            ProjectMessage::Add(path) => {
                let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
                let path = metadata
                    .packages
                    .iter()
                    .map(|package| package.manifest_path.parent().unwrap())
                    .find(|crate_path| path.starts_with(crate_path))
                    .unwrap()
                    .to_path_buf()
                    .into_std_path_buf();
                self.projects.entry(path).or_insert(ProjectData {
                    status: ProjectStatus::BeforeCompile,
                });
                iced::Task::none()
            }
            ProjectMessage::Remove(path) => {
                self.projects.shift_remove(&path);
                iced::Task::none()
            }
            ProjectMessage::Compile(path) => {
                if let Some(state) = self.projects.get_mut(&path) {
                    *&mut state.status = ProjectStatus::OnCompile;
                    iced::Task::perform(
                        Self::compile(path.clone(), scene_settings.clone()),
                        move |result| ProjectMessage::CompileResult(path.clone(), result),
                    )
                } else {
                    iced::Task::none()
                }
            }
            ProjectMessage::CompileResult(path, compile_result) => {
                if let Some(state) = self.projects.get_mut(&path) {
                    *&mut state.status = match compile_result {
                        Err(error) => ProjectStatus::CompileError(error),
                        Ok(scenes) => ProjectStatus::AfterCompile {scenes, active: None},
                    };
                }
                iced::Task::none()
            }
        }
    }

    async fn compile(
        path: PathBuf,
        scene_settings: Arc<SceneSettings>,
    ) -> anyhow::Result<indexmap::IndexMap<String, SceneData>> {
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
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let mut buf = String::new();
        let mut scenes = indexmap::IndexMap::new();
        while stdout.read_line(&mut buf)? != 0 {
            let (name, scene_data): (String, SceneData) = ron::de::from_str(&buf)?;
            scenes.insert(name, scene_data);
            buf.clear();
        }
        Ok(scenes)
    }

    // fn prepare(
    //     timeline_entries: Arc<TimelineEntries>,
    //     device: Arc<wgpu::Device>,
    // ) -> anyhow::Result<PresentationEntries> {
    //     timeline_entries.prepare(&device)
    // }
}

#[derive(Debug)]
pub enum ProjectMessage {
    Add(PathBuf),
    Remove(PathBuf),
    Compile(PathBuf),
    CompileResult(
        PathBuf,
        anyhow::Result<indexmap::IndexMap<String, SceneData>>,
    ),
    // Prepare(PathBuf, String),
    // PrepareResult(PathBuf, String, anyhow::Result<PresentationEntries>),
    // PresentError(PathBuf, String, anyhow::Error),
}

struct ProjectData {
    status: ProjectStatus,
}

enum ProjectStatus {
    BeforeCompile,
    OnCompile,
    AfterCompile {
        scenes: indexmap::IndexMap<String, SceneData>,
        active: Option<String>
    },
    CompileError(anyhow::Error),
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

#[derive(Clone, Copy)]
enum ProgressSpeed {
    Forward050,
    Forward075,
    Forward100,
    Forward125,
    Forward150,
    Forward200,
    Backward050,
    Backward075,
    Backward100,
    Backward125,
    Backward150,
    Backward200,
}

impl ProgressSpeed {
    fn value(&self) -> f32 {
        match self {
            Self::Forward050 => 0.50,
            Self::Forward075 => 0.75,
            Self::Forward100 => 1.00,
            Self::Forward125 => 1.25,
            Self::Forward150 => 1.50,
            Self::Forward200 => 2.00,
            Self::Backward050 => -0.50,
            Self::Backward075 => -0.75,
            Self::Backward100 => -1.00,
            Self::Backward125 => -1.25,
            Self::Backward150 => -1.50,
            Self::Backward200 => -2.00,
        }
    }

    fn display_str(&self) -> &'static str {
        match self {
            Self::Forward050 => "0.5x",
            Self::Forward075 => "0.75x",
            Self::Forward100 => "speed",
            Self::Forward125 => "1.25x",
            Self::Forward150 => "1.5x",
            Self::Forward200 => "2x",
            Self::Backward050 => "-0.5x",
            Self::Backward075 => "-0.75",
            Self::Backward100 => "-1x",
            Self::Backward125 => "-1.25",
            Self::Backward150 => "-1.5x",
            Self::Backward200 => "-2x",
        }
    }
}

struct Progress {
    time_interval: Range<f32>,
    // anchor_time: f32,
    // instant: Instant,
    time: f32,
    // base_speed: f32,
    progress_speed: ProgressSpeed,
    paused: bool,
}

impl Progress {
    fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            time: 0.0,
            // instant: Instant::now(),
            // base_speed,
            progress_speed: ProgressSpeed::Forward100,
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
            self.time += app_delta_time * self.progress_speed.value();
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

// impl Default for Progress {
//     fn default() -> Self {
//         Self::new(0.0, 1.0)
//     }
// }

#[derive(Debug)]
pub(crate) struct Primitive(Option<(TimelineEntries, f32)>);

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
        if let Some((timeline_entries, time)) = &self.0 {
            timeline_entries.prepare(*time, device, queue, format, storage, bounds, viewport);
        }
    }

    fn render(
        &self,
        encoder: &mut iced::widget::shader::wgpu::CommandEncoder,
        storage: &iced::widget::shader::Storage,
        target: &iced::widget::shader::wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        {
            let mut render_pass =
                encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        iced::widget::shader::wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops: iced::widget::shader::wgpu::Operations {
                                load: iced::widget::shader::wgpu::LoadOp::Clear(
                                    iced::widget::shader::wgpu::Color::TRANSPARENT,
                                ),
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
        if let Some((timeline_entries, time)) = &self.0 {
            timeline_entries.render(*time, encoder, storage, target, clip_bounds);
        }
    }
}

#[derive(Default)]
pub(crate) struct State {
    scene_settings: Arc<SceneSettings>,
    player_settings: PlayerSettings,
    project_manager: ProjectManager,
    // progress: Progress,
    // active_scene: Option<SceneData>,
}

impl State {
    pub(crate) fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ProjectMessage(project_message) => self
                .project_manager
                .update(project_message, self.scene_settings.clone())
                .map(Message::ProjectMessage),
        }
    }

    pub(crate) fn view(&self) -> iced::Element<Message> {
        iced::widget::Shader::new(self).into()
    }
}

impl iced::widget::shader::Program<Message> for State {
    type State = ();
    type Primitive = Primitive;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::shader::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        shell: &mut iced::advanced::Shell<'_, Message>,
    ) -> (iced::event::Status, Option<Message>) {
        if self.project_manager.active.is_some() && !self.progress.paused() {
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
        Primitive(
            self.active_scene
                .as_ref()
                .map(|active_scene| (active_scene.timeline_entries.clone(), self.progress.time)),
        )
    }
}

#[derive(Debug)]
pub(crate) enum Message {
    ProjectMessage(ProjectMessage),
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
