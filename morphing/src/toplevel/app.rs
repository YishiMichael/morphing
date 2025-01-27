// pub trait Scene {
//     fn construct(self, sv: &Supervisor<'_>);

//     fn override_settings(&self, scene_settings: SceneSettings) -> SceneSettings {
//         scene_settings
//     }
// }

// pub trait Scenes {
//     fn run(self);
// }

// impl<S> Scenes for S
// where
//     S: Scene,
// {
//     fn run(self) {
//         let scene_settings = read_from_stdin();
//         let scene_timelines = SceneTimelines::new(0, scene_settings, self);
//         write_to_stdout(scene_timelines);
//     }
// }

// macro_rules! scenes_tuple {
//     ($($i:tt,)*) => {paste::paste!{
//         impl<$(${ignore($i)} [<S${index()}>],)*> Scenes for ($(${ignore($i)} [<S${index()}>],)*)
//         where
//             $(${ignore($i)} [<S${index()}>]: 'static + Send + Scene,)*
//         {
//             #[allow(unused_variables)]
//             fn run(self) {
//                 let scene_settings: SceneSettings = read_from_stdin();
//                 $(${ignore($i)} let [<thread_${index()}>] = {
//                     let scene_settings = scene_settings.clone();
//                     std::thread::spawn(move || {
//                         let scene_timelines = SceneTimelines::new(${index()}, scene_settings, self.${index()});
//                         write_to_stdout(scene_timelines);
//                     })
//                 };)*
//                 $(${ignore($i)} [<thread_${index()}>].join().unwrap();)*
//             }
//         }
//     }};
// }
// scenes_tuple!();
// scenes_tuple!(_,);
// scenes_tuple!(_, _,);
// scenes_tuple!(_, _, _,);
// scenes_tuple!(_, _, _, _,);
// scenes_tuple!(_, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _, _,);
// scenes_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _,);
// }

pub use ron;

pub mod storyboard {
    use std::future::Future;
    use std::ops::RangeFrom;
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::super::super::timelines::timeline::PresentationEntries;
    use super::super::super::timelines::timeline::TimelineEntries;
    use super::super::scene::SceneTimelineCollection;
    use super::super::settings::VideoSettings;

    pub(crate) struct StoryboardManager {
        storyboards: Vec<StoryboardState>,
        storyboard_id_counter: RangeFrom<u32>,
    }

    impl StoryboardManager {
        pub(crate) fn update(
            &mut self,
            message: StoryboardMessage,
            device: Arc<wgpu::Device>,
        ) -> iced::Task<StoryboardMessage> {
            match message {
                StoryboardMessage::Compile(path) => {
                    let storyboard_id = StoryboardId(self.storyboard_id_counter.next().unwrap());
                    self.storyboards.push(StoryboardState {
                        id: storyboard_id,
                        path: path.clone(),
                        status: StoryboardStatus::Compile,
                    });
                    iced::Task::perform(Self::compile(path), move |result| {
                        StoryboardMessage::Execute(storyboard_id, result)
                    })
                }
                StoryboardMessage::Execute(storyboard_id, compile_result) => {
                    if let Some(status) = self
                        .storyboards
                        .iter_mut()
                        .find(|state| state.id == storyboard_id)
                        .map(|state| &mut state.status)
                    {
                        match compile_result {
                            Err(err) => {
                                *status = StoryboardStatus::CompileError(err);
                                iced::Task::none()
                            }
                            Ok(path) => {
                                *status = StoryboardStatus::Execute;
                                iced::Task::perform(Self::execute(path), move |result| {
                                    StoryboardMessage::Precut(storyboard_id, result)
                                })
                            }
                        }
                    } else {
                        iced::Task::none()
                    }
                }
                StoryboardMessage::Precut(storyboard_id, execute_result) => {
                    if let Some(status) = self
                        .storyboards
                        .iter_mut()
                        .find(|state| state.id == storyboard_id)
                        .map(|state| &mut state.status)
                    {
                        match execute_result {
                            Err(err) => {
                                *status = StoryboardStatus::ExecuteError(err);
                                iced::Task::none()
                            }
                            Ok(scene_timelines) => {
                                let mut scene_states = Vec::new();
                                let mut timeline_entries_collection = Vec::new();
                                for scene_timeline in scene_timelines {
                                    scene_states.push(SceneState {
                                        id: SceneId(0),
                                        name: scene_timeline.name.to_string(),
                                        video_settings: scene_timeline.video_settings,
                                        duration: scene_timeline.duration,
                                        status: SceneStatus::Precut,
                                    });
                                    timeline_entries_collection
                                        .push(scene_timeline.timeline_entries);
                                }
                                *status = StoryboardStatus::Success(scene_states);
                                iced::Task::batch(timeline_entries_collection.into_iter().map(
                                    |timeline_entries| {
                                        let device = device.clone();
                                        iced::Task::perform(
                                            Self::precut(timeline_entries, device),
                                            move |result| {
                                                StoryboardMessage::Present(
                                                    storyboard_id,
                                                    SceneId(0),
                                                    result,
                                                )  // TODO
                                            },
                                        )
                                    },
                                ))
                            }
                        }
                    } else {
                        iced::Task::none()
                    }
                }
                StoryboardMessage::Present(storyboard_id, timeline_id, precut_result) => {
                    if let Some(StoryboardStatus::Success(scene_states)) = self
                        .storyboards
                        .iter_mut()
                        .find(|state| state.id == storyboard_id)
                        .map(|state| &mut state.status)
                    {
                        if let Some(status) = scene_states
                            .iter_mut()
                            .find(|state| state.id == timeline_id)
                            .map(|state| &mut state.status)
                        {
                            match precut_result {
                                Err(err) => {
                                    *status = SceneStatus::PrecutError(err);
                                    iced::Task::none()
                                }
                                Ok(presentation_entries) => {
                                    *status = SceneStatus::Success(presentation_entries);
                                    iced::Task::none()
                                }
                            }
                        } else {
                            iced::Task::none()
                        }
                    } else {
                        iced::Task::none()
                    }
                }
            }
        }

        fn compile(_path: PathBuf) -> impl Future<Output = anyhow::Result<PathBuf>> {
            async {
                todo!()
            }
        }

        fn execute(_path: PathBuf) -> impl Future<Output = anyhow::Result<Vec<SceneTimelineCollection>>> {
            async {
                todo!()
            }
        }

        fn precut(
            timeline_entries: TimelineEntries,
            device: Arc<wgpu::Device>,
        ) -> impl Future<Output = anyhow::Result<PresentationEntries>> {
            async {
                todo!()
            }
            // async move {
            //     timeline_entries.precut(&device)
            // }
        }
    }

    pub(crate) enum StoryboardMessage {
        Compile(PathBuf),
        Execute(StoryboardId, anyhow::Result<PathBuf>),
        Precut(StoryboardId, anyhow::Result<Vec<SceneTimelineCollection>>),
        Present(StoryboardId, SceneId, anyhow::Result<PresentationEntries>),
        // PresentError(StoryboardId, SceneId, anyhow::Error),
    }

    struct StoryboardState {
        id: StoryboardId,
        path: PathBuf,
        status: StoryboardStatus,
    }

    #[derive(Clone, Copy, PartialEq)]
    struct StoryboardId(u32);

    enum StoryboardStatus {
        Compile,
        Execute,
        Success(Vec<SceneState>),
        CompileError(anyhow::Error),
        ExecuteError(anyhow::Error),
    }

    struct SceneState {
        id: SceneId,
        name: String,
        video_settings: VideoSettings,
        duration: f32,
        status: SceneStatus,
    }

    #[derive(Clone, Copy, PartialEq)]
    struct SceneId(u32);

    enum SceneStatus {
        Precut,
        Success(PresentationEntries),
        PrecutError(anyhow::Error),
        PresentError(anyhow::Error),
    }
}

pub mod app {
    use std::ops::Range;
    use std::sync::Arc;

    use super::super::super::toplevel::settings::Settings;
    use super::storyboard::StoryboardManager;
    use super::storyboard::StoryboardMessage;

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

    // struct ActiveScene {
    //     progress: Progress,
    //     scene: Arc<SceneState>,
    // }
    struct State {
        settings: Settings,
        // active_scene: Option<ActiveScene>,
        device: Arc<wgpu::Device>,
        storyboard_manager: StoryboardManager,
        // window: Option<Arc<winit::window::Window>>,
        // renderer: OnceLock<Renderer>,
        // progress: Progress,
        // control_pressed: bool,
        // presentation_collection: Option<PresentationCollection>,
    }

    impl State {
        fn update(&mut self, message: Message) -> iced::Task<Message> {
            match message {
                Message::StoryboardMessage(storyboard_message) => self
                    .storyboard_manager
                    .update(storyboard_message, self.device.clone())
                    .map(Message::StoryboardMessage),
            }
        }
    }

    enum Message {
        StoryboardMessage(StoryboardMessage),
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
