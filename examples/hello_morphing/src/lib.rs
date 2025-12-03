// use morphing_core::scene::export_scenes;
// use morphing_core::scene::scene;
// use morphing_core::scene::SceneFilter;
// use morphing_core::timeline::Supervisor;
// use morphing_gizmos::mobjects::shape::Rect;

use morphing_core::{chapter, scene, Supervisor};

// #[scene]
// fn demo_scene(sv: &Supervisor<'_>) {
//     sv.wait(1.0);
//     let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
//     sv.wait(6.0);
//     drop(mobject);
//     sv.wait(12.0);
// }

// #[scene(config = "my_config.toml")]
// fn another_demo_scene(sv: &Supervisor<'_>) {
//     sv.wait(1.0);
//     let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
//     sv.wait(6.0);
//     drop(mobject);
//     sv.wait(10.0);
// }

#[chapter(config(toml = "", toml = "", yamla = ""))]
extern crate self;

#[derive(serde::Deserialize)]
struct MyConfig {}

// #[scene(config = toml, )]
fn my_scene(sv: &mut Supervisor<MyConfig>) {
    sv.with(TriangleLifecycle {});
}
