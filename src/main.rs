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

use notify::Watcher;
use std::path::Path;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let src_path = Path::new("../demo/src/").canonicalize().unwrap();
    // Set up the file system watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    let res = watcher.configure(notify::Config::default().with_compare_contents(true));
    println!("{res:?}");
    watcher
        .watch(&src_path, notify::RecursiveMode::Recursive)
        .unwrap();

    // let binary_path = src_path.join("../target/debug").canonicalize().unwrap();
    // let mut child: Option<Child> = None;
    // start_child_process(&binary_path)?;

    for res in rx {
        match res {
            Ok(event) if matches!(event.kind, notify::EventKind::Modify(..)) => {
                // Recompile the child binary
                println!("Recompiling...");
                match Command::new("cargo")
                    .arg("run")
                    .current_dir(src_path.clone())
                    .output()
                {
                    Ok(output) => {
                        let string = String::from_utf8(output.stdout).unwrap();
                        println!("Got string: {string}");
                    }
                    Err(_) => {
                        println!("Failed to recompile");
                    }
                }
            }
            Ok(_) => (),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
