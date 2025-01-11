use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use super::renderer::Renderer;

pub trait Present: 'static {
    fn present(&self, time: f32, time_interval: Range<f32>, renderer: &Renderer);
}

pub(crate) struct ArchivedPresentations(Vec<(Range<Arc<f32>>, Box<dyn Present>)>);

impl ArchivedPresentations {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn present_all(&self, time: f32, renderer: &Renderer) {
        for (time_range, presentation) in &self.0 {
            let time_range = *time_range.start..*time_range.end;
            if time_range.contains(&time) {
                presentation.present(time, time_range, renderer);
            }
        }
    }
}

pub struct Supervisor {
    time: Arc<f32>,
    presentations: RefCell<ArchivedPresentations>,
}

impl Supervisor {
    pub(crate) fn new() -> Self {
        Self {
            time: Arc::new(0.0),
            presentations: RefCell::new(ArchivedPresentations::new()),
        }
    }

    pub(crate) fn into_presentations(self) -> ArchivedPresentations {
        self.presentations.into_inner()
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.time.clone()
    }

    pub(crate) fn archive_presentation<P>(&self, time_interval: Range<Arc<f32>>, presentation: P)
    where
        P: Present,
    {
        if !Arc::<f32>::ptr_eq(&time_interval.start, &time_interval.end) {
            self.presentations
                .borrow_mut()
                .0
                .push((time_interval, Box::new(presentation)));
        }
    }

    pub fn wait(&mut self, delta_time: f32) {
        assert!(delta_time.is_sign_positive());
        self.time = Arc::new(*self.time + delta_time);
    }
}

// use std::collections::HashMap;

// use itertools::Itertools;
// use serde::Deserialize;
// use serde::Serialize;

// use super::world::WORLD;

// use std::sync::Arc;

// use super::timelines::timeline::Supervisor;

// pub trait Scene {
//     fn construct(self, supervisor: &Supervisor);
// }

// pub fn run<S>(scene: S)
// where
//     S: Scene,
// {
//     let supervisor = Supervisor::new();
//     scene.construct(&supervisor);
//     let supervisor_data = supervisor.into_data();
// }

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
