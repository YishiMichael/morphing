use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use super::app::App;
use super::config::Config;
use super::renderer::Renderer;
use super::world::World;

pub trait Present: 'static {
    fn present(&self, time: f32, time_interval: Range<f32>, renderer: &Renderer);
}

pub(crate) struct PresentationCollection {
    time: Arc<f32>,
    presentations: Vec<(Range<Arc<f32>>, Box<dyn Present>)>,
}

impl PresentationCollection {
    pub(crate) fn new() -> Self {
        Self {
            time: Arc::new(0.0),
            presentations: Vec::new(),
        }
    }

    pub(crate) fn full_time(&self) -> f32 {
        *self.time
    }

    pub(crate) fn present_all(&self, time: f32, renderer: &Renderer) {
        for (time_range, presentation) in &self.presentations {
            let time_range = *time_range.start..*time_range.end;
            if time_range.contains(&time) {
                presentation.present(time, time_range, renderer);
            }
        }
    }
}

pub struct Supervisor {
    world: Arc<World>,
    presentation_collection: RefCell<PresentationCollection>,
}

impl Supervisor {
    pub(crate) fn new(world: Arc<World>) -> Self {
        Self {
            world,
            presentation_collection: RefCell::new(PresentationCollection::new()),
        }
    }

    pub(crate) fn world(&self) -> &Arc<World> {
        &self.world
    }

    pub(crate) fn into_collection(self) -> PresentationCollection {
        self.presentation_collection.into_inner()
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.presentation_collection.borrow().time.clone()
    }

    pub(crate) fn archive_presentation<P>(&self, time_interval: Range<Arc<f32>>, presentation: P)
    where
        P: Present,
    {
        if !Arc::<f32>::ptr_eq(&time_interval.start, &time_interval.end) {
            self.presentation_collection
                .borrow_mut()
                .presentations
                .push((time_interval, Box::new(presentation)));
        }
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative argument `delta_time`, got {delta_time}",
        );
        let time = &mut self.presentation_collection.borrow_mut().time;
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

pub trait Scene: Sized {
    fn construct(self, supervisor: &Supervisor);

    fn run(self, config: Config) -> anyhow::Result<()> {
        let supervisor = Supervisor::new(Arc::new(World::new(config.style, config.typst)));
        self.construct(&supervisor);
        App::instantiate_and_run(supervisor.into_collection(), config.window, config.video)?;
        Ok(())
    }
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
