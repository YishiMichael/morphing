use std::collections::HashMap;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use super::world::WORLD;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Worldline {
    data: u32,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct BakedWorldline {
    data: String,
}

impl Worldline {
    fn bake(&self) -> BakedWorldline {
        // demo baking: 3 |-> "0,1,2"
        println!("Baking... {}", self.data);
        BakedWorldline {
            data: (0..self.data).map(|i| i.to_string()).join(","),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Scene {
    worldlines: Vec<Worldline>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            worldlines: Vec::new(),
        }
    }

    pub fn push_worldline(&mut self, worldline: u32) -> &mut Self {
        self.worldlines.push(Worldline { data: worldline });
        self
    }

    pub fn run(&self) {
        self.bake().render();
    }

    fn bake(&self) -> BakedScene {
        let in_cache = WORLD.read_cache();
        let mut out_cache = HashMap::new();
        let baked_scene = BakedScene {
            baked_worldlines: self
                .worldlines
                .iter()
                .map(|worldline| {
                    let baked_worldline = in_cache
                        .get(&worldline)
                        .cloned()
                        .unwrap_or_else(|| worldline.bake());
                    out_cache.insert(worldline.clone(), baked_worldline.clone());
                    baked_worldline
                })
                .collect(),
        };
        WORLD.write_cache(out_cache);
        baked_scene
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct BakedScene {
    baked_worldlines: Vec<BakedWorldline>,
}

impl BakedScene {
    fn render(self) {
        self.baked_worldlines
            .into_iter()
            .for_each(|baked_worldline| println!("{:?}", baked_worldline.data));
    }
}
