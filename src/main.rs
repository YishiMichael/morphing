pub mod world;

use world::TypstWorld;

fn main() -> () {
    let world = TypstWorld::default();

    let text = "text".to_string();

    world.document(text);

    // println!("{content:?}");
    // println!("{document:?}");
    // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
    // println!("{svg}");
}
