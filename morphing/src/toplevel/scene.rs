use std::io::Read;

pub use inventory;
pub use morphing_macros::scene;

use super::super::timelines::timeline::Supervisor;
use super::super::timelines::timeline::TimelineEntries;
use super::settings::SceneSettings;
use super::settings::VideoSettings;
use super::world::World;

pub struct SceneModule {
    pub name: &'static str,
    pub override_settings: Option<fn(SceneSettings) -> SceneSettings>,
    pub scene_fn: fn(&Supervisor),
}

inventory::collect!(SceneModule);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SceneData {
    pub timeline_collection: Option<SceneTimelineCollection>,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SceneTimelineCollection {
    pub time: f32,
    pub timeline_entries: TimelineEntries,
    pub video_settings: VideoSettings,
}

pub fn export_scenes() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
    let mut shh_stdout = shh::stdout().unwrap();
    let mut shh_stderr = shh::stderr().unwrap();
    for scene_module in inventory::iter::<SceneModule>() {
        let name = scene_module.name.to_string();
        let timeline_collection = std::panic::catch_unwind(|| {
            let scene_settings = if let Some(override_settings) = scene_module.override_settings {
                override_settings(scene_settings.clone())
            } else {
                scene_settings.clone()
            };
            Supervisor::visit(
                &World::new(scene_settings.style, scene_settings.typst),
                scene_module.scene_fn,
                |time, timeline_entries, _| SceneTimelineCollection {
                    time,
                    timeline_entries,
                    video_settings: scene_settings.video,
                },
            )
        })
        .ok();
        let mut stdout_bytes = Vec::new();
        shh_stdout.read_to_end(&mut stdout_bytes).unwrap();
        let mut stderr_bytes = Vec::new();
        shh_stderr.read_to_end(&mut stderr_bytes).unwrap();
        let scene_data = SceneData {
            timeline_collection,
            stdout_bytes,
            stderr_bytes,
        };
        println!("{}", ron::ser::to_string(&(name, scene_data)).unwrap());
    }
}
