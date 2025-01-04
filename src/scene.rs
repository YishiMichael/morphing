use std::collections::HashMap;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Worldline {
    data: u32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BakedWorldline {
    data: String,
}

impl Worldline {
    pub(crate) fn bake(&self) -> BakedWorldline {
        // demo baking: 3 |-> "0,1,2"
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

    pub(crate) fn bake(&self) -> BakedScene {
        BakedScene {
            baked_worldlines: self.worldlines.iter().map(Worldline::bake).collect(),
        }
    }

    pub(crate) fn bake_with_cache(
        &self,
        in_cache: &HashMap<Worldline, BakedWorldline>,
        out_cache: &mut HashMap<Worldline, BakedWorldline>,
    ) -> BakedScene {
        BakedScene {
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
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct BakedScene {
    baked_worldlines: Vec<BakedWorldline>,
}

impl BakedScene {
    pub(crate) fn render(self) -> anyhow::Result<()> {
        self.baked_worldlines
            .into_iter()
            .for_each(|baked_worldline| println!("{:?}", baked_worldline.data));
        Ok(())
    }
}
