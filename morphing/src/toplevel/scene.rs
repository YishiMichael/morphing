use std::cell::RefCell;
use std::io::Read;
use std::ops::Range;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

use super::super::timelines::timeline::Presentation;
use super::super::timelines::timeline::Timeline;
use super::settings::SceneSettings;
use super::settings::VideoSettings;
use super::world::World;

pub use morphing_macros::scene;

pub struct Supervisor<'w> {
    world: &'w World,
    time: RefCell<Arc<f32>>,
    timeline_entries: RefCell<Vec<TimelineEntry>>,
}

impl<'w> Supervisor<'w> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            time: RefCell::new(Arc::new(0.0)),
            timeline_entries: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn into_timeline_collection(self) -> TimelineCollection {
        TimelineCollection {
            time: *self.time.into_inner(),
            timeline_entries: self.timeline_entries.into_inner(),
        }
    }

    pub(crate) fn world(&self) -> &'w World {
        self.world
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub(crate) fn archive_timeline<T>(&self, time_interval: Range<Arc<f32>>, timeline: T)
    where
        T: Timeline,
    {
        if !Arc::<f32>::ptr_eq(&time_interval.start, &time_interval.end) {
            let time_interval = *time_interval.start..*time_interval.end;
            self.timeline_entries.borrow_mut().push(TimelineEntry {
                time_interval,
                timeline: Box::new(timeline),
            });
        }
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative argument `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

struct PresentationEntry<'t> {
    time_interval: Range<f32>,
    presentation: Box<dyn 't + Presentation>,
}

pub(crate) struct PresentationCollection<'t> {
    time: f32,
    presentation_entries: Vec<PresentationEntry<'t>>,
}

impl PresentationCollection<'_> {
    pub(crate) fn time(&self) -> f32 {
        self.time
    }

    pub(crate) fn present_collection(
        &self,
        time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
    ) {
        for PresentationEntry {
            time_interval,
            presentation,
        } in &self.presentation_entries
        {
            if time_interval.contains(&time) {
                presentation.present(time, time_interval.clone(), device, queue, render_pass);
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TimelineEntry {
    time_interval: Range<f32>,
    #[serde(with = "serde_traitobject")]
    timeline: Box<dyn Timeline>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TimelineCollection {
    time: f32,
    timeline_entries: Vec<TimelineEntry>,
}

impl TimelineCollection {
    pub(crate) fn presentation_collection<'t>(
        &'t self,
        device: &wgpu::Device,
    ) -> PresentationCollection<'t> {
        PresentationCollection {
            time: self.time,
            presentation_entries: self
                .timeline_entries
                .iter()
                .map(
                    |TimelineEntry {
                         time_interval,
                         timeline,
                     }| PresentationEntry {
                        time_interval: time_interval.clone(),
                        presentation: timeline.presentation(device),
                    },
                )
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SceneTimelineCollectionModule {
    name: &'static str,
    video_settings: VideoSettings,
    timeline_collection: TimelineCollection,
}

impl SceneTimelineCollectionModule {
    pub fn new<S>(name: &'static str, scene_settings: SceneSettings, scene_fn: S) -> Self
    where
        S: FnOnce(&Supervisor),
    {
        let world = World::new(scene_settings.style, scene_settings.typst);
        let supervisor = Supervisor::new(&world);
        scene_fn(&supervisor);
        Self {
            name,
            video_settings: scene_settings.video,
            timeline_collection: supervisor.into_timeline_collection(),
        }
    }
}

pub fn execute<S>(scene: S)
where
    S: FnOnce(SceneSettings) -> SceneTimelineCollectionModule,
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
//     let module = SceneTimelineCollectionModule {
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
