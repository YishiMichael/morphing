use std::collections::HashMap;

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

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct SceneTimelineCollection {
    pub(crate) name: String,
    pub(crate) video_settings: VideoSettings,
    pub(crate) duration: f32,
    pub(crate) timeline_entries: TimelineEntries,
}

pub fn export_scenes() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
    let mut override_settings_map = HashMap::new();
    for scene_module in inventory::iter::<SceneModule>() {
        let (video_settings, world) = override_settings_map
            .entry(scene_module.override_settings)
            .or_insert_with(|| {
                let scene_settings = if let Some(override_settings) = scene_module.override_settings
                {
                    override_settings(scene_settings.clone())
                } else {
                    scene_settings.clone()
                };
                (
                    scene_settings.video,
                    World::new(scene_settings.style, scene_settings.typst),
                )
            });
        let supervisor = Supervisor::new(world);
        (scene_module.scene_fn)(&supervisor);
        let scene_timeline_collection = SceneTimelineCollection {
            name: scene_module.name.to_string(),
            video_settings: video_settings.clone(),
            duration: *supervisor.time(),
            timeline_entries: supervisor.into_timeline_entries(),
        };
        println!(
            "{}",
            ron::ser::to_string(&scene_timeline_collection).unwrap()
        );
    }
}
