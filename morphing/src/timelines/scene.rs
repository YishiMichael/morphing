use std::io::Read;

use super::super::toplevel::settings::SceneSettings;
use super::super::toplevel::settings::VideoSettings;
use super::super::toplevel::world::World;
use super::alive::Supervisor;
use super::timeline::TimelineEntries;

pub use morphing_macros::scene;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SceneTimelines {
    pub(crate) name: String,
    pub(crate) video_settings: VideoSettings,
    pub(crate) duration: f32,
    pub(crate) timeline_entries: TimelineEntries,
}

impl SceneTimelines {
    pub fn new<S>(name: &'static str, scene_settings: SceneSettings, scene_fn: S) -> Self
    where
        S: FnOnce(&Supervisor),
    {
        let world = World::new(scene_settings.style, scene_settings.typst);
        let supervisor = Supervisor::new(&world);
        scene_fn(&supervisor);
        Self {
            name: String::from(name),
            video_settings: scene_settings.video,
            duration: *supervisor.get_time(),
            timeline_entries: supervisor.into_timeline_entries(),
        }
    }
}

// pub(crate) struct ScenePresentations {
//     pub(crate) name: &'static str,
//     pub(crate) video_settings: VideoSettings,
//     pub(crate) duration: f32,
//     pub(crate) presentation_entries: PresentationEntries,
// }

// impl ScenePresentations {
//     pub(crate) fn new(timelines: SceneTimelines, device: &wgpu::Device) -> Self {
//         let SceneTimelines {
//             name,
//             video_settings,
//             duration,
//             timeline_entries,
//         } = timelines;
//         Self {
//             name,
//             video_settings,
//             duration,
//             presentation_entries: timeline_entries.presentation(device),
//         }
//     }
// }

// TODO: execute_parallel
pub fn execute<S>(scene: S)
where
    S: FnOnce(SceneSettings) -> SceneTimelines,
{
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    let scene_settings = ron::de::from_str(&buf).unwrap();
    let module = scene(scene_settings);
    println!("{}", ron::ser::to_string(&module).unwrap());
}

// pub fn read_app_scene_settings() -> SceneSettings {
//     let (_, settings) = std::env::args().collect_tuple().unwrap();
//     ron::de::from_str(&settings).unwrap()
// }

// pub fn write_scene_timelines<S>(name: String, scene: S, scene_settings: SceneSettings)
// where
//     S: Scene,
// {
//     let module = SceneTimelines {
//         name,
//         video_settings: scene_settings.video,
//         timeline_collection: supervisor.into_timeline_collection(),
//     };
//     println!("{}", ron::ser::to_string(&module).unwrap());
// }

// pub fn run() {}

// #[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub struct Worldline {
//     data: u32,
// }

// #[derive(Clone, Deserialize, Serialize)]
// pub struct BakedWorldline {
//     data: String,
// }

// impl Worldline {
//     fn bake(&self) -> BakedWorldline {
//         // demo baking: 3 |-> "0,1,2"
//         println!("Baking... {}", self.data);
//         BakedWorldline {
//             data: (0..self.data).map(|i| i.to_string()).join(","),
//         }
//     }
// }

// #[derive(Deserialize, Serialize)]
// pub struct Scene {
//     worldlines: Vec<Worldline>,
// }

// impl Scene {
//     pub fn new() -> Self {
//         Self {
//             worldlines: Vec::new(),
//         }
//     }

//     pub fn push_worldline(&mut self, worldline: u32) -> &mut Self {
//         self.worldlines.push(Worldline { data: worldline });
//         self
//     }

//     pub fn run(&self) {
//         self.bake().render();
//     }

//     fn bake(&self) -> BakedScene {
//         let in_cache = WORLD.read_cache();
//         let mut out_cache = HashMap::new();
//         let baked_scene = BakedScene {
//             baked_worldlines: self
//                 .worldlines
//                 .iter()
//                 .map(|worldline| {
//                     let baked_worldline = in_cache
//                         .get(&worldline)
//                         .cloned()
//                         .unwrap_or_else(|| worldline.bake());
//                     out_cache.insert(worldline.clone(), baked_worldline.clone());
//                     baked_worldline
//                 })
//                 .collect(),
//         };
//         WORLD.write_cache(out_cache);
//         baked_scene
//     }
// }

// #[derive(Deserialize, Serialize)]
// pub struct BakedScene {
//     baked_worldlines: Vec<BakedWorldline>,
// }

// impl BakedScene {
//     fn render(self) {
//         self.baked_worldlines
//             .into_iter()
//             .for_each(|baked_worldline| println!("{:?}", baked_worldline.data));
//     }
// }
