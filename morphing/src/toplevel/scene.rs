use super::super::timelines::alive::Supervisor;
use super::super::timelines::timeline::TimelineEntries;
use super::settings::SceneSettings;
use super::settings::VideoSettings;
use super::world::World;

pub use morphing_macros::scene;

pub struct SceneTimelines {
    pub name: &'static str,
    pub video_settings: VideoSettings,
    pub duration: f32,
    pub timeline_entries: TimelineEntries,
} // TODO: remove internal pubs

impl SceneTimelines {
    pub fn new<S>(name: &'static str, scene_settings: SceneSettings, scene_fn: S) -> Self
    where
        S: Fn(&Supervisor),
    {
        let world = World::new(scene_settings.style, scene_settings.typst);
        let supervisor = Supervisor::new(&world);
        scene_fn(&supervisor);
        Self {
            name,
            video_settings: scene_settings.video,
            duration: *supervisor.get_time(),
            timeline_entries: supervisor.into_timeline_entries(),
        }
    }
}
