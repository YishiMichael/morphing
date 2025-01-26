use super::super::timelines::alive::Supervisor;
use super::super::timelines::timeline::TimelineEntries;
use super::settings::SceneSettings;
use super::settings::VideoSettings;
use super::world::World;

pub use morphing_macros::scene;

pub struct SceneTimelines {
    pub(crate) id: usize,
    pub(crate) name: &'static str,
    pub(crate) video_settings: VideoSettings,
    pub(crate) duration: f32,
    pub(crate) timeline_entries: TimelineEntries,
}

impl SceneTimelines {
    pub fn new<S>(id: usize, name: &'static str, scene_settings: SceneSettings, scene_fn: S) -> Self
    where
        S: Fn(&Supervisor),
    {
        let world = World::new(scene_settings.style, scene_settings.typst);
        let supervisor = Supervisor::new(&world);
        scene_fn(&supervisor);
        Self {
            id,
            name,
            video_settings: scene_settings.video,
            duration: *supervisor.get_time(),
            timeline_entries: supervisor.into_timeline_entries(),
        }
    }
}
