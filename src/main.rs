// use crate::typst::typst_mobject;

// fn main() -> () {
//     let mobs = typst_mobject("typst \\ text text #[text] text $ a b c - d^2 $");
//     // let mobs = typst_mobject("fish \\ #[f]ish");
//     for (_mobject, _transform, span) in mobs {
//         dbg!(span);
//     }

//     // frame.items().for_each(|a|);

//     // println!("{content:?}");
//     // println!("{document:?}");
//     // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
//     // println!("{svg}");
// }

use clap::Parser;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let cli = morphing::cli::Cli::parse();
    // $ cargo run -- watch ../demo/src
    match cli.subcommand {
        morphing::cli::CliSubcommand::Run { path } => morphing::cli::run(
            path.unwrap_or(Path::new(".").to_path_buf())
                .canonicalize()?,
        ),
        morphing::cli::CliSubcommand::Watch { path } => morphing::cli::watch(
            path.unwrap_or(Path::new(".").to_path_buf())
                .canonicalize()?,
        ),
    }?;
    Ok(())
}
