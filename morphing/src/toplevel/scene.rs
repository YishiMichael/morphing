use std::collections::HashMap;

pub use inventory;
use itertools::Itertools;
pub use morphing_macros::scene;

use super::super::timelines::alive::Supervisor;
use super::super::timelines::timeline::TimelineEntries;
use super::io::read_from_stdin;
use super::io::write_to_stdout;
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
    pub(crate) name: &'static str,
    pub(crate) video_settings: VideoSettings,
    pub(crate) duration: f32,
    pub(crate) timeline_entries: TimelineEntries,
}

pub fn export_scenes() {
    let scene_module_inventory = inventory::iter::<SceneModule>().collect_vec();
    write_to_stdout(
        scene_module_inventory
            .iter()
            .map(|scene_module| scene_module.name.to_string())
            .collect_vec(),
    );
    write_to_stdout((
        SceneSettings::default(),
        vec!["abc".to_string(), "def".to_string()],
    )); // TODO
    let (scene_settings, selected_scene_names): (SceneSettings, Vec<String>) = read_from_stdin();
    let mut override_settings_map = HashMap::new();
    for selected_scene_name in selected_scene_names {
        let scene_module = scene_module_inventory
            .iter()
            .find(|scene_module| scene_module.name == selected_scene_name.as_str())
            .unwrap();
        let (world, video_settings) = override_settings_map
            .entry(scene_module.override_settings)
            .or_insert_with(|| {
                let scene_settings = if let Some(override_settings) = scene_module.override_settings
                {
                    override_settings(scene_settings.clone())
                } else {
                    scene_settings.clone()
                };
                let world = World::new(scene_settings.style, scene_settings.typst);
                (world, scene_settings.video)
            });
        let supervisor = Supervisor::new(world);
        (scene_module.scene_fn)(&supervisor);
        write_to_stdout(SceneTimelineCollection {
            name: scene_module.name,
            video_settings: video_settings.clone(),
            duration: *supervisor.get_time(),
            timeline_entries: supervisor.into_timeline_entries(),
        });
    }
}
