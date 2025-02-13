use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::path::PathBuf;
use std::process::ChildStdout;

use morphing_core::config::Config;
use morphing_core::scene::read_and_deserialize;
use morphing_core::scene::LineOutput;

use super::message::AppMessage;
use super::message::ProgressMessage;
use super::message::ProjectStateMessage;
use super::message::ProjectSuccessStateMessage;
use super::message::SceneStateMessage;
use super::message::SceneSuccessStateMessage;
use super::state::AppState;
use super::state::LogLevel;
use super::state::Logger;
use super::state::Progress;
use super::state::ProjectState;
use super::state::ProjectSuccessState;
use super::state::SceneState;
use super::state::SceneSuccessState;

pub(crate) fn update(state: &mut AppState, message: AppMessage) -> iced::Task<AppMessage> {
    state.update(message)
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

    fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Menu => iced::Task::none(),
            // AppMessage::SetDefaultSceneSettings(scene_settings) => {
            //     self.scene_settings = Arc::new(scene_settings);
            //     iced::Task::none()
            // }
            AppMessage::Open => iced::Task::perform(pick_folders(), AppMessage::OpenReply),
            AppMessage::OpenReply(reply) => {
                dbg!(reply.clone());
                match reply {
                    Some(paths) => iced::Task::batch(paths.into_iter().map(|path| {
                        iced::Task::done(AppMessage::ProjectState(
                            path,
                            ProjectStateMessage::Compile,
                        ))
                    })), // TODO: verify they are crate folders
                    None => iced::Task::none(),
                }
            }
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
                    generation: 0,
                })
                .update(message)
                .map(move |message| AppMessage::ProjectState(path.clone(), message)),
        }
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
                        Ok(lines) => iced::Task::stream(lines).map(|line| match line {
                            Ok(line) => match read_and_deserialize(&line) {
                                Ok(line_output) => match line_output {
                                    LineOutput::Project(project_redirected_output) => {
                                        ProjectStateMessage::ReloadProject(
                                            project_redirected_output,
                                        )
                                    }
                                    LineOutput::Scene(name, scene_redirected_output) => {
                                        ProjectStateMessage::ProjectSuccessState(
                                            ProjectSuccessStateMessage::SceneState(
                                                name,
                                                SceneStateMessage::ReloadScene(
                                                    scene_redirected_output,
                                                ),
                                            ),
                                        )
                                    }
                                },
                                Err(error) => ProjectStateMessage::CompileError(error.to_string()),
                            },
                            Err(error) => ProjectStateMessage::CompileError(error.to_string()),
                        }),
                        Err(error) => {
                            iced::Task::done(ProjectStateMessage::CompileError(error.to_string()))
                        }
                    }
                })
            }
            ProjectStateMessage::CompileError(error) => {
                self.logger.log(LogLevel::Error, error);
                iced::Task::none()
            }
            ProjectStateMessage::ReloadProject(project_redirected_output) => {
                for line in project_redirected_output.stdout_lines.iter() {
                    self.logger.log(LogLevel::Info, line);
                }
                for line in project_redirected_output.stderr_lines.iter() {
                    self.logger.log(LogLevel::Error, line);
                }
                match project_redirected_output.result {
                    Ok(()) => {
                        self.generation += 1;
                        self.logger.log(
                            LogLevel::Trace,
                            format!("Project reloaded [generation #{}]", self.generation),
                        );
                    }
                    Err(()) => {
                        self.logger.log(
                            LogLevel::Error,
                            "Failed to reload project (see errors above)",
                        );
                    }
                }
                iced::Task::none()
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
                    generation: 0,
                })
                .update(message)
                .map(move |message| ProjectSuccessStateMessage::SceneState(name.clone(), message)),
        }
    }
}

impl SceneState {
    fn update(&mut self, message: SceneStateMessage) -> iced::Task<SceneStateMessage> {
        match message {
            SceneStateMessage::ReloadScene(scene_redirected_output) => {
                for line in scene_redirected_output.stdout_lines.iter() {
                    self.logger.log(LogLevel::Info, line);
                }
                for line in scene_redirected_output.stderr_lines.iter() {
                    self.logger.log(LogLevel::Error, line);
                }
                match scene_redirected_output.result {
                    Ok(Some(scene_data)) => {
                        self.generation += 1;
                        self.logger.log(
                            LogLevel::Trace,
                            format!("Scene reloaded [generation #{}]", self.generation),
                        );
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

impl Progress {
    fn update(&mut self, message: ProgressMessage) -> iced::Task<ProgressMessage> {
        self.refresh_anchor();
        match message {
            ProgressMessage::SetTime(time) => {
                self.time = time;
            }
            ProgressMessage::SetSpeed(speed) => {
                self.speed = speed;
            }
            ProgressMessage::SetPlayDirection(play_direction) => {
                self.play_direction = play_direction;
            }
            ProgressMessage::SetPlaying(playing) => {
                self.playing = playing;
            }
        }
        iced::Task::none()
    }
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(PathBuf::from)
}

async fn pick_folders() -> Option<Vec<PathBuf>> {
    rfd::AsyncFileDialog::new()
        .pick_folders()
        .await
        .map(|folders| folders.into_iter().map(PathBuf::from).collect())
}

async fn pick_save_file(filter_name: &str, filter_extensions: &[&str]) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(filter_name, filter_extensions)
        .save_file()
        .await
        .map(PathBuf::from)
}

async fn compile_project(
    path: PathBuf,
) -> std::io::Result<futures::stream::Iter<Lines<BufReader<ChildStdout>>>> {
    // TODO: check if synchronous streaming works
    let mut child = std::process::Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    Ok(futures::stream::iter(
        BufReader::new(child.stdout.take().unwrap()).lines(),
    ))
}
