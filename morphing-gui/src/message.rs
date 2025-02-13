use std::path::PathBuf;

use morphing_core::scene::RedirectedOutput;
use morphing_core::scene::SceneData;

use super::state::PlayDirection;

#[derive(Clone, Debug)]
pub enum AppMessage {
    Menu, // Sentinel message to activate menu buttons
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
    ReloadProject(RedirectedOutput<()>),
    ProjectSuccessState(ProjectSuccessStateMessage),
}

#[derive(Clone, Debug)]
pub enum ProjectSuccessStateMessage {
    SaveVideos,
    SaveVideosReply(Option<PathBuf>),
    Activate(Option<String>),
    SceneState(String, SceneStateMessage),
}

#[derive(Clone, Debug)]
pub enum SceneStateMessage {
    ReloadScene(RedirectedOutput<Option<SceneData>>),
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

#[derive(Clone, Debug)]
pub enum ProgressMessage {
    SetTime(f32),
    SetSpeed(f32),
    SetPlayDirection(PlayDirection),
    SetPlaying(bool),
}
