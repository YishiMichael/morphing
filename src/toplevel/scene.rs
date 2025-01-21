use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use super::super::timelines::timeline::Timeline;
use super::world::World;

trait DynTimeline {
    fn dyn_presentation<'t>(&'t self, device: &wgpu::Device) -> Box<dyn Presentation + 't>;
}

impl<T> DynTimeline for T
where
    T: Timeline,
{
    fn dyn_presentation<'t>(&'t self, device: &wgpu::Device) -> Box<dyn Presentation + 't> {
        Box::new(self.presentation(device))
    }
}

pub trait Presentation {
    fn present(
        &self,
        time: f32,
        time_interval: Range<f32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
    );
}

pub(crate) struct PresentationCollection<'t> {
    time: f32,
    presentations: Vec<(Range<f32>, Box<dyn Presentation + 't>)>,
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
        for (time_interval, presentation) in &self.presentations {
            if time_interval.contains(&time) {
                presentation.present(time, time_interval.clone(), device, queue, render_pass);
            }
        }
    }
}

pub(crate) struct TimelineCollection {
    time: f32,
    timelines: Vec<(Range<f32>, Box<dyn DynTimeline>)>,
}

impl TimelineCollection {
    pub(crate) fn presentation_collection<'t>(
        &'t self,
        device: &wgpu::Device,
    ) -> PresentationCollection<'t> {
        PresentationCollection {
            time: self.time,
            presentations: self
                .timelines
                .iter()
                .map(|(time_interval, timeline)| {
                    (time_interval.clone(), timeline.dyn_presentation(device))
                })
                .collect(),
        }
    }
}

pub struct Supervisor<'w> {
    world: &'w World,
    time: RefCell<Arc<f32>>,
    timelines: RefCell<Vec<(Range<f32>, Box<dyn DynTimeline>)>>,
}

impl<'w> Supervisor<'w> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            time: RefCell::new(Arc::new(0.0)),
            timelines: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn world(&self) -> &'w World {
        self.world
    }

    pub(crate) fn into_collection(self) -> TimelineCollection {
        TimelineCollection {
            time: *self.time.into_inner(),
            timelines: self.timelines.into_inner(),
        }
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub(crate) fn archive_timeline<T>(&self, time_interval: Range<Arc<f32>>, timeline: T)
    where
        T: Timeline + 'static,
    {
        if !Arc::<f32>::ptr_eq(&time_interval.start, &time_interval.end) {
            let time_interval = *time_interval.start..*time_interval.end;
            self.timelines
                .borrow_mut()
                .push((time_interval, Box::new(timeline)));
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

// use std::collections::HashMap;

// use itertools::Itertools;
// use serde::Deserialize;
// use serde::Serialize;

// use super::world::WORLD;

// use std::sync::Arc;

// use super::timelines::timeline::Supervisor;

pub trait Scene: Default {
    fn size() -> Option<(u32, u32)> {
        None
    }

    fn background_color() -> Option<palette::Srgba> {
        None
    }

    fn fps() -> Option<f32> {
        None
    }

    fn play_speed() -> Option<f32> {
        None
    }

    fn construct(self, supervisor: &Supervisor);

    // fn run(self, config: Config) -> anyhow::Result<()> {
    //     let world = World::new(config.style, config.typst);
    //     let supervisor = Supervisor::new(&world);
    //     self.construct(&supervisor);
    //     App::instantiate_and_run(supervisor.into_collection(), config.window, config.video)?;
    //     Ok(())
    // }
}

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
