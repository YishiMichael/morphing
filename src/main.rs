pub mod fill;
pub mod mobject;
pub mod path;
pub mod stroke;
pub mod typst;

use crate::typst::typst_mobject;

fn main() -> () {
    let mobs = typst_mobject("typst \\ text");
    for (_mobject, transform, span) in mobs {
        dbg!((transform, span));
    }

    // frame.items().for_each(|a|);

    // println!("{content:?}");
    // println!("{document:?}");
    // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
    // println!("{svg}");
}
