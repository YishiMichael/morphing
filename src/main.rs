pub mod world;

use world::TypstWorld;

fn main() {
    let mut world = TypstWorld::default();
    world.set_text("text".to_string());

    let document = typst::compile(&world);
    dbg!(document.warnings);
}
