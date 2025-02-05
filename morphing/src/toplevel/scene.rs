use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;

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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SceneData {
    pub result: SceneResult,
    pub stdout_lines: Arc<Vec<String>>,
    pub stderr_lines: Arc<Vec<String>>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SceneResult {
    Success {
        time: f32,
        timeline_entries: TimelineEntries,
        video_settings: VideoSettings,
    },
    Error,
    Skipped,
}

pub fn export_scenes_by_regex(regex: &lazy_regex::Regex) {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
    let mut override_settings_map = HashMap::new();

    for scene_module in inventory::iter::<SceneModule>() {
        let shh_stdout = BufReader::new(shh::stdout().unwrap());
        let shh_stderr = BufReader::new(shh::stderr().unwrap());
        let name = scene_module.name.to_string();
        let (world, video_settings) = override_settings_map
            .entry(scene_module.override_settings)
            .or_insert_with_key(|override_settings| {
                let scene_settings = override_settings
                    .and_then(|override_settings| {
                        std::panic::catch_unwind(|| override_settings(scene_settings.clone()))
                            .inspect_err(|_| {
                                eprintln!(
                                "`override_settings` panicked. Use default scene settings instead"
                            )
                            })
                            .ok()
                    })
                    .unwrap_or(scene_settings.clone());
                (
                    Mutex::new(World::new(scene_settings.style, scene_settings.typst)),
                    scene_settings.video,
                )
            });
        let result = if regex.is_match(&name) {
            std::panic::catch_unwind(|| {
                Supervisor::visit(
                    &world.lock().unwrap(),
                    scene_module.scene_fn,
                    |time, timeline_entries, _| SceneResult::Success {
                        time,
                        timeline_entries,
                        video_settings: video_settings.clone(),
                    },
                )
            })
            .unwrap_or(SceneResult::Error)
        } else {
            SceneResult::Skipped
        };
        let scene_data = SceneData {
            result,
            stdout_lines: Arc::new(shh_stdout.lines().map(Result::unwrap).collect()),
            stderr_lines: Arc::new(shh_stderr.lines().map(Result::unwrap).collect()),
        };
        println!("{}", ron::ser::to_string(&(name, scene_data)).unwrap());
    }
}

pub fn export_scenes() {
    export_scenes_by_regex(lazy_regex::regex!(".*"));
}
