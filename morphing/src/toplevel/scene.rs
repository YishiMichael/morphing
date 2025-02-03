use std::io::BufRead;
use std::io::BufReader;

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
    pub name: String,
    pub result: SceneResult,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum SceneResult {
    Success {
        time: f32,
        timeline_entries: TimelineEntries,
        video_settings: VideoSettings,
    },
    Error,
    Skipped,
}

pub fn export_scenes(regex: Option<&lazy_regex::Regex>) {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();

    for scene_module in inventory::iter::<SceneModule>() {
        let shh_stdout = BufReader::new(shh::stdout().unwrap());
        let shh_stderr = BufReader::new(shh::stderr().unwrap());
        let name = scene_module.name.to_string();
        let result = if regex.is_none_or(|regex| regex.is_match(&name)) {
            std::panic::catch_unwind(|| {
                let scene_settings = if let Some(override_settings) = scene_module.override_settings
                {
                    override_settings(scene_settings.clone())
                } else {
                    scene_settings.clone()
                };
                Supervisor::visit(
                    &World::new(scene_settings.style, scene_settings.typst),
                    scene_module.scene_fn,
                    |time, timeline_entries, _| SceneResult::Success {
                        time,
                        timeline_entries,
                        video_settings: scene_settings.video,
                    },
                )
            })
            .unwrap_or(SceneResult::Error)
        } else {
            SceneResult::Skipped
        };
        let scene_data = SceneData {
            name,
            result,
            stdout_lines: shh_stdout.lines().map(Result::unwrap).collect(),
            stderr_lines: shh_stderr.lines().map(Result::unwrap).collect(),
        };
        println!("{}", ron::ser::to_string(&scene_data).unwrap());
    }
}
