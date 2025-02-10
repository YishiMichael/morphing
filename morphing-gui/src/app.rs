use std::path::PathBuf;

use morphing::timelines::timeline::TimelineEntries;
use morphing::toplevel::config::Config;
use morphing::toplevel::scene::ProjectRedirectedResult;
use morphing::toplevel::scene::SceneData;
use morphing::toplevel::scene::SceneRedirectedResult;

use super::collection::Collection;
use super::collection::CollectionItem;
use super::io::compile_project;
use super::io::pick_folder;
use super::io::pick_folders;
use super::io::pick_save_file;
use super::logger::LogLevel;
use super::logger::Logger;
use super::progress::Progress;
use super::progress::ProgressMessage;

#[derive(Debug, Default)]
pub(crate) struct AppState {
    projects: Collection<ProjectState>,
}

#[derive(Debug)]
struct ProjectState {
    path: PathBuf,
    watching: bool, // TODO
    project_success_state: Option<ProjectSuccessState>,
    logger: Logger,
}

#[derive(Debug, Default)]
struct ProjectSuccessState {
    generation: usize,
    scenes: Collection<SceneState>,
}

#[derive(Debug)]
struct SceneState {
    name: String,
    scene_success_state: Option<SceneSuccessState>,
    logger: Logger,
}

#[derive(Debug, Default)]
struct SceneSuccessState {
    generation: usize,
    progress: Progress,
    timeline_entries: TimelineEntries,
    config: Config,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    Menu, // Sentinel message to activate menu buttons
    // SetDefaultSceneSettings(SceneSettings),
    Open,
    OpenReply(Option<Vec<PathBuf>>),
    Close(PathBuf),
    Activate(Option<PathBuf>),
    ProjectState(PathBuf, ProjectStateMessage),
}

#[derive(Clone, Debug)]
pub enum ProjectStateMessage {
    SetPath(PathBuf),
    Compile,
    CompileError(String),
    SetWatching(bool),
    ReloadProject(ProjectRedirectedResult),
    ProjectSuccessState(ProjectSuccessStateMessage),
}

#[derive(Clone, Debug)]
pub enum ProjectSuccessStateMessage {
    ReloadProjectSuccess,
    SaveVideos,
    SaveVideosReply(Option<PathBuf>),
    Activate(Option<String>),
    SceneState(String, SceneStateMessage),
}

#[derive(Clone, Debug)]
pub enum SceneStateMessage {
    ReloadScene(SceneRedirectedResult),
    SceneSuccessState(SceneSuccessStateMessage),
}

#[derive(Clone, Debug)]
pub enum SceneSuccessStateMessage {
    // Reload(SceneData),
    // SetVideoSettings(VideoSettings),
    ReloadSceneSuccess(SceneData),
    SaveVideo,
    SaveVideoReply(Option<PathBuf>),
    SaveImage,
    SaveImageReply(Option<PathBuf>),
    Progress(ProgressMessage),
}

impl CollectionItem for ProjectState {
    type Key = PathBuf;

    fn key(&self) -> &Self::Key {
        &self.path
    }
}

impl CollectionItem for SceneState {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }
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
            AppMessage::Menu => iced::Task::none(),
            // AppMessage::SetDefaultSceneSettings(scene_settings) => {
            //     self.scene_settings = Arc::new(scene_settings);
            //     iced::Task::none()
            // }
            AppMessage::Open => iced::Task::perform(pick_folders(), AppMessage::OpenReply),
            AppMessage::OpenReply(reply) => match reply {
                Some(paths) => iced::Task::batch(paths.into_iter().map(|path| {
                    iced::Task::done(AppMessage::ProjectState(path, ProjectStateMessage::Compile))
                })),
                None => iced::Task::none(),
            },
            AppMessage::Close(path) => {
                self.projects.remove(&path);
                iced::Task::none()
            }
            AppMessage::Activate(path) => {
                self.projects.set_active(path.as_ref());
                iced::Task::none()
            }
            AppMessage::ProjectState(path, message) => self
                .projects
                .active_find_or_insert_with(path.clone(), |path| ProjectState {
                    path,
                    watching: false,
                    project_success_state: None,
                    logger: Logger::default(),
                })
                .update(message)
                .map(move |message| AppMessage::ProjectState(path.clone(), message)),
        }
    }

    pub(crate) fn view(&self) -> iced::Element<AppMessage> {
        let menu_bar = {
            fn menu_bar_style(
                theme: &iced::Theme,
                _status: iced_aw::style::Status,
            ) -> iced_aw::widget::menu::Style {
                let palette = theme.extended_palette();
                iced_aw::widget::menu::Style {
                    bar_background: palette.background.base.color.into(),
                    bar_border: iced::Border::default(),
                    bar_shadow: iced::Shadow::default(),
                    bar_background_expand: iced::Padding::default(),
                    menu_background: palette.background.base.color.into(),
                    menu_border: iced::Border::default()
                        .width(1.0)
                        .color(palette.background.strong.color),
                    menu_shadow: iced::Shadow::default(),
                    menu_background_expand: iced::Padding::default(),
                    path: iced::Color::default().into(),
                    path_border: iced::Border::default(),
                }
                // match status {
                //     iced::widget::button::Status::Active => iced::widget::button::Style {
                //         background: None,
                //         text_color: pair.text,
                //         ..Default::default()
                //     },
                //     iced::widget::button::Status::Hovered
                //     | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                //         background: Some(iced::Background::Color(pair.color)),
                //         text_color: pair.text,
                //         ..Default::default()
                //     },
                //     iced::widget::button::Status::Disabled => iced::widget::button::Style {
                //         background: None,
                //         text_color: pair.text.scale_alpha(0.3),
                //         ..Default::default()
                //     },
                // }
            }

            fn menu_button_style(
                theme: &iced::Theme,
                status: iced::widget::button::Status,
            ) -> iced::widget::button::Style {
                let palette = theme.extended_palette();
                match status {
                    iced::widget::button::Status::Active => iced::widget::button::Style {
                        background: None,
                        text_color: palette.secondary.base.text,
                        ..Default::default()
                    },
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(iced::Background::Color(palette.secondary.weak.color)),
                        text_color: palette.secondary.base.text,
                        ..Default::default()
                    },
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: None,
                        text_color: palette.secondary.base.text.scale_alpha(0.3),
                        ..Default::default()
                    },
                }
            }

            fn menu_button(text: &str) -> iced::widget::Button<AppMessage> {
                iced::widget::button(iced::widget::text(text).size(14.0))
                    .padding([1.0, 6.0])
                    .style(menu_button_style)
            }

            let open_message = Some(AppMessage::Open);
            let close_message = self
                .projects
                .get_active()
                .map(|project_state| AppMessage::Close(project_state.path.clone()));
            let save_video_message = self.projects.get_active().and_then(|project_state| {
                project_state
                    .project_success_state
                    .as_ref()
                    .and_then(|project_success_state| {
                        project_success_state
                            .scenes
                            .get_active()
                            .and_then(|scene_state| {
                                scene_state.scene_success_state.is_some().then_some(
                                    AppMessage::ProjectState(
                                        project_state.path.clone(),
                                        ProjectStateMessage::ProjectSuccessState(
                                            ProjectSuccessStateMessage::SceneState(
                                                scene_state.name.clone(),
                                                SceneStateMessage::SceneSuccessState(
                                                    SceneSuccessStateMessage::SaveVideo,
                                                ),
                                            ),
                                        ),
                                    ),
                                )
                            })
                    })
            });
            let save_image_message = self.projects.get_active().and_then(|project_state| {
                project_state
                    .project_success_state
                    .as_ref()
                    .and_then(|project_success_state| {
                        project_success_state
                            .scenes
                            .get_active()
                            .and_then(|scene_state| {
                                scene_state.scene_success_state.is_some().then_some(
                                    AppMessage::ProjectState(
                                        project_state.path.clone(),
                                        ProjectStateMessage::ProjectSuccessState(
                                            ProjectSuccessStateMessage::SceneState(
                                                scene_state.name.clone(),
                                                SceneStateMessage::SceneSuccessState(
                                                    SceneSuccessStateMessage::SaveVideo,
                                                ),
                                            ),
                                        ),
                                    ),
                                )
                            })
                    })
            });

            iced_aw::menu::MenuBar::new(Vec::from([
                iced_aw::menu::Item::with_menu(
                    menu_button("File").on_press(AppMessage::Menu),
                    iced_aw::menu::Menu::new(Vec::from([
                        iced_aw::menu::Item::new(
                            menu_button("Open")
                                .width(iced::Length::Fill)
                                .on_press_maybe(open_message),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Close")
                                .width(iced::Length::Fill)
                                .on_press_maybe(close_message),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Save Video")
                                .width(iced::Length::Fill)
                                .on_press_maybe(save_video_message),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Save Image")
                                .width(iced::Length::Fill)
                                .on_press_maybe(save_image_message),
                        ),
                    ]))
                    .width(180.0)
                    .offset(2.0),
                ),
                iced_aw::menu::Item::with_menu(
                    menu_button("Setting").on_press(AppMessage::Menu),
                    iced_aw::menu::Menu::new(Vec::from([
                        iced_aw::menu::Item::with_menu(
                            menu_button("Default Scene Settings")
                                .width(iced::Length::Fill)
                                .on_press(AppMessage::Menu),
                            iced_aw::menu::Menu::new(Vec::from([
                                iced_aw::menu::Item::new(
                                    menu_button("Open")
                                        .width(iced::Length::Fill)
                                        .on_press(AppMessage::Menu),
                                ),
                                iced_aw::menu::Item::new(
                                    menu_button("Close")
                                        .width(iced::Length::Fill)
                                        .on_press(AppMessage::Menu),
                                ),
                                iced_aw::menu::Item::new(
                                    menu_button("Save Video")
                                        .width(iced::Length::Fill)
                                        .on_press(AppMessage::Menu),
                                ),
                                iced_aw::menu::Item::new(
                                    menu_button("Save Image")
                                        .width(iced::Length::Fill)
                                        .on_press(AppMessage::Menu),
                                ),
                            ]))
                            .width(180.0)
                            .offset(2.0),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Video Settings").width(iced::Length::Fill),
                        ),
                    ]))
                    .width(180.0)
                    .offset(2.0),
                ),
            ]))
            .style(menu_bar_style)
        };

        iced::widget::Column::new().push(menu_bar).into()

        // iced::widget::Shader::new(self).into()
    }

    pub(crate) fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

impl ProjectState {
    fn update(&mut self, message: ProjectStateMessage) -> iced::Task<ProjectStateMessage> {
        match message {
            ProjectStateMessage::SetPath(path) => {
                self.path = path;
                iced::Task::none()
            }
            ProjectStateMessage::Compile => {
                self.logger.log(LogLevel::Trace, "Compilation starts");
                iced::Task::future(compile_project(self.path.clone())).then(|io_result| {
                    match io_result {
                        Ok(Ok((project_redirected_result, scene_stream))) => {
                            iced::Task::done(ProjectStateMessage::ReloadProject(
                                project_redirected_result,
                            ))
                            .chain(
                                iced::Task::stream(scene_stream).map(
                                    |(name, scene_redirected_result)| {
                                        ProjectStateMessage::ProjectSuccessState(
                                            ProjectSuccessStateMessage::SceneState(
                                                name,
                                                SceneStateMessage::ReloadScene(
                                                    scene_redirected_result,
                                                ),
                                            ),
                                        )
                                    },
                                ),
                            )

                            // self.logger.log(LogLevel::Trace, "Compilation ends");
                        }
                        Ok(Err(compile_error)) => {
                            // self.logger.log(LogLevel::Error, compile_error);
                            // self.logger.log(LogLevel::Trace, "Compilation fails");
                            iced::Task::done(ProjectStateMessage::CompileError(compile_error))
                        }
                        Err(io_error) => {
                            // self.logger.log(LogLevel::Error, io_error.to_string());
                            // self.logger.log(LogLevel::Trace, "Compilation fails");
                            iced::Task::done(ProjectStateMessage::CompileError(
                                io_error.to_string(),
                            ))
                        }
                    }
                })
                // .then(|lines| {
                //     iced::Task::none() // TODO
                // })
            }
            ProjectStateMessage::CompileError(error) => {
                self.logger.log(LogLevel::Error, error);
                self.logger.log(LogLevel::Trace, "Compilation fails");
                iced::Task::none()
            }
            ProjectStateMessage::ReloadProject(project_redirected_result) => {
                for line in project_redirected_result.stdout_lines.iter() {
                    self.logger.log(LogLevel::Info, line);
                }
                for line in project_redirected_result.stderr_lines.iter() {
                    self.logger.log(LogLevel::Error, line);
                }
                match project_redirected_result.result {
                    Ok(()) => {
                        self.logger.log(LogLevel::Trace, "Project reloaded");
                        iced::Task::done(ProjectStateMessage::ProjectSuccessState(
                            ProjectSuccessStateMessage::ReloadProjectSuccess,
                        ))
                    }
                    Err(()) => {
                        self.logger.log(
                            LogLevel::Error,
                            "Failed to reload project (see errors above)",
                        );
                        iced::Task::none()
                    }
                }
            }
            // ProjectStateMessage::CompileResult(result) => match result {
            //     Ok(scenes_data) => {
            //         self.logger.log(LogLevel::Trace, "Compilation ends");
            //         iced::Task::batch(scenes_data.into_iter().map(|(name, scene_data)| {
            //             iced::Task::done(ProjectStateMessage::ProjectSuccessState(
            //                 ProjectSuccessStateMessage::SceneState(
            //                     name,
            //                     SceneStateMessage::Reload(scene_data),
            //                 ),
            //             ))
            //         }))
            //     }
            //     Err(error) => {
            //         self.logger.log(LogLevel::Error, error);
            //         self.logger.log(LogLevel::Trace, "Compilation fails");
            //         iced::Task::none()
            //     }
            // },
            ProjectStateMessage::SetWatching(watching) => {
                self.watching = watching;
                todo!();
                iced::Task::none()
            }
            ProjectStateMessage::ProjectSuccessState(message) => self
                .project_success_state
                .get_or_insert_default()
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
            ProjectSuccessStateMessage::ReloadProjectSuccess => {
                self.generation += 1;
                iced::Task::none()
            }
            ProjectSuccessStateMessage::SaveVideos => {
                iced::Task::perform(pick_folder(), ProjectSuccessStateMessage::SaveVideosReply)
            }
            ProjectSuccessStateMessage::SaveVideosReply(reply) => match reply {
                Some(path) => iced::Task::batch(self.scenes.iter().map(|scene| {
                    let name = scene.name.clone();
                    iced::Task::done(ProjectSuccessStateMessage::SceneState(
                        name.clone(),
                        SceneStateMessage::SceneSuccessState(
                            SceneSuccessStateMessage::SaveVideoReply(Some(
                                path.join(format!("{name}.mp4")),
                            )),
                        ),
                    ))
                })),
                None => iced::Task::none(),
            },
            ProjectSuccessStateMessage::Activate(name) => {
                self.scenes.set_active(name.as_ref());
                iced::Task::none()
            }
            ProjectSuccessStateMessage::SceneState(name, message) => self
                .scenes
                .inactive_find_or_insert_with(name.clone(), |name| SceneState {
                    name,
                    scene_success_state: None,
                    logger: Logger::default(),
                })
                .update(message)
                .map(move |message| ProjectSuccessStateMessage::SceneState(name.clone(), message)),
        }
    }
}

impl SceneState {
    fn update(&mut self, message: SceneStateMessage) -> iced::Task<SceneStateMessage> {
        match message {
            SceneStateMessage::ReloadScene(scene_redirected_result) => {
                for line in scene_redirected_result.stdout_lines.iter() {
                    self.logger.log(LogLevel::Info, line);
                }
                for line in scene_redirected_result.stderr_lines.iter() {
                    self.logger.log(LogLevel::Error, line);
                }
                match scene_redirected_result.result {
                    Ok(Some(scene_data)) => {
                        self.logger.log(LogLevel::Trace, "Scene reloaded");
                        iced::Task::done(SceneStateMessage::SceneSuccessState(
                            SceneSuccessStateMessage::ReloadSceneSuccess(scene_data),
                        ))
                    }
                    Ok(None) => {
                        self.logger.log(LogLevel::Trace, "Scene skipped");
                        iced::Task::none()
                    }
                    Err(()) => {
                        self.logger
                            .log(LogLevel::Error, "Failed to reload scene (see errors above)");
                        iced::Task::none()
                    }
                }
            }
            SceneStateMessage::SceneSuccessState(message) => self
                .scene_success_state
                .get_or_insert_default()
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
            SceneSuccessStateMessage::ReloadSceneSuccess(scene_data) => {
                self.generation += 1;
                self.progress = Progress::new(scene_data.time);
                self.timeline_entries = scene_data.timeline_entries;
                self.config = Config::new(scene_data.config_values);
                iced::Task::none()
            }
            // SceneSuccessStateMessage::SetVideoSettings(video_settings) => {
            //     self.video_settings = video_settings;
            //     iced::Task::none()
            // }
            SceneSuccessStateMessage::SaveVideo => iced::Task::perform(
                pick_save_file("MP4", &["mp4"]),
                SceneSuccessStateMessage::SaveVideoReply,
            ),
            SceneSuccessStateMessage::SaveVideoReply(reply) => {
                todo!();
                iced::Task::none()
            }
            SceneSuccessStateMessage::SaveImage => iced::Task::perform(
                pick_save_file("PNG", &["png"]),
                SceneSuccessStateMessage::SaveImageReply,
            ),
            SceneSuccessStateMessage::SaveImageReply(reply) => {
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

#[derive(Debug)]
struct ScenePrimitive {
    time: f32,
    timeline_entries: TimelineEntries,
    size: (u32, u32),
    fps: f64,
    background_color: iced::widget::shader::wgpu::Color,
}

impl iced::widget::shader::Primitive for ScenePrimitive {
    fn prepare(
        &self,
        device: &iced::widget::shader::wgpu::Device,
        queue: &iced::widget::shader::wgpu::Queue,
        format: iced::widget::shader::wgpu::TextureFormat,
        storage: &mut iced::widget::shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        self.timeline_entries
            .prepare(self.time, device, queue, format, storage, bounds, viewport);
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
                                    self.background_color,
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
        self.timeline_entries
            .render(self.time, encoder, storage, target, clip_bounds);
    }
}

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
