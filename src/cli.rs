use clap::Parser;
use clap::Subcommand;
use notify::Watcher;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread::JoinHandle;

use super::scene::BakedScene;
use super::scene::BakedWorldline;
use super::scene::Scene;
use super::scene::Worldline;

#[derive(Parser)]
#[command(name = "morph", version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: CliSubcommand,
}

#[derive(Subcommand)]
pub enum CliSubcommand {
    Run { path: Option<PathBuf> },
    Watch { path: Option<PathBuf> },
    // SaveVideo { output: PathBuf, input: PathBuf },
    // SaveImage { output: PathBuf, input: PathBuf },  // rfd
    // https://docs.rs/rfd/latest/rfd/struct.FileDialog.html
    // #[command(hide = true)]
    // InternalCompile,
    // #[command(hide = true)]
    // InternalBake,
    // #[command(hide = true)]
    // InternalBakeIncremental,
    // #[command(hide = true)]
    // InternalDisplay,
}

// fn internal_compile() -> std::io::Result<Child> {
//     Command::new(env!("CARGO_CRATE_NAME"))
//         .arg("internal-compile")
//         .spawn()
// }

// fn internal_bake() -> std::io::Result<Child> {
//     Command::new(env!("CARGO_CRATE_NAME"))
//         .arg("internal-bake")
//         .spawn()
// }

// fn internal_bake_incremental() -> std::io::Result<Child> {
//     Command::new(env!("CARGO_CRATE_NAME"))
//         .arg("internal-bake-incremental")
//         .spawn()
// }

// fn internal_display() -> std::io::Result<Child> {
//     Command::new(env!("CARGO_CRATE_NAME"))
//         .arg("internal-display")
//         .spawn()
// }

// #[derive(Deserialize, Serialize)]
// struct BakeInput {
//     path: PathBuf,
// }

// #[derive(Deserialize, Serialize)]
// struct BakeOutput {
//     baked_scene: BakedScene,
// }

// #[derive(Deserialize, Serialize)]
// struct BakeIncrementalInput {
//     path: PathBuf,
//     cache: String,
// }

// #[derive(Deserialize, Serialize)]
// struct BakeIncrementalOutput {
//     baked_scene: BakedScene,
//     cache: String,
// }

// struct ProcessWrapper(Child);

// impl ProcessWrapper {
//     fn bake_input(path: PathBuf) -> anyhow::Result<Self> {
//         // TODO: pipes needed?
//         let mut process = Command::new(env!("CARGO_CRATE_NAME"))
//             .arg("internal-bake")
//             // .arg("--path")
//             // .arg(path)
//             .spawn()?;
//         ron::ser::to_writer(process.stdin.as_mut().unwrap(), &BakeInput { path })?;
//         process.kill()
//         Ok(Self(process))
//     }

//     // fn bake_output(self) ->
// }

// fn internal_bake(path: PathBuf) -> BakedScene {}

// struct BakeOutput {}

// fn read_from_stdin_and_deserialize<T: serde::Deserialize>() -> anyhow::Result<()> {
//     let mut stdin = std::io::stdin();
//     ron::de::from_reader(rdr)
//     let mut buf = Vec::new();
//     stdin.read_to_end(buf)?;

//     Ok(())
// }

// fn serialize_and_write_to_stdout<T: serde::Serialize>(value: &T) -> ron::Result<()> {
//     ron::ser::to_writer(std::io::stdout(), value)
// }

// fn run_scene(path: &Path) -> anyhow::Result<()> {
//     println!("#1");
//     let output = Command::new("cargo")
//         .arg("run")
//         .current_dir(path)
//         .output()?;
//     println!("#2");
//     let string = String::from_utf8(output.stdout)?;
//     println!("{string}");
//     Ok(())
// }

pub fn run(path: PathBuf) -> anyhow::Result<()> {
    let scene: Scene = ron::de::from_reader(
        Command::new("cargo")
            .arg("run")
            .stdout(Stdio::piped())
            .current_dir(path)
            .output()?
            .stdout
            .as_slice(),
    )?;
    let baked_scene = scene.bake();
    baked_scene.render()?;

    // // TODO: pipes needed?
    // let internal_compile = internal_compile()?;
    // ron::ser::to_writer(internal_compile.stdin.as_mut().unwrap(), &path)?;
    // let scene_bytes = internal_compile.wait_with_output()?.stdout;
    // let internal_bake = internal_bake()?;
    // internal_bake.stdin.as_mut().unwrap().write(&scene_bytes);
    // let internal_display = internal_display()?;

    // let process = Command::new(env!("CARGO_CRATE_NAME"))
    //     .arg("internal-bake")
    //     // .arg("--path")
    //     // .arg(path)
    //     .spawn()?;

    // // let output: BakeOutput = ron::de::from_bytes(&process.wait_with_output()?.stdout)?;
    // // output.baked_scene.render()?;
    Ok(())
}

pub fn watch(path: PathBuf) -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let cache_path = tempdir.path().join("cache");
    let baked_scene_path = tempdir.path().join("baked_scene");
    let cache_tempfile = File::create_new(cache_path)?;
    let baked_scene_tempfile = File::create_new(baked_scene_path.clone())?;

    // let (source_tx, source_rx) = std::sync::mpsc::channel();
    // let mut source_watcher = notify::recommended_watcher(source_tx)?;
    // source_watcher.watch(&path, notify::RecursiveMode::Recursive)?;
    // let (scene_tx, scene_rx) = std::sync::mpsc::channel();
    // let mut scene_watcher = notify::recommended_watcher(scene_tx)?;
    // scene_watcher.watch(&scene_path, notify::RecursiveMode::Recursive)?;
    // let (cache_tx, cache_rx) = std::sync::mpsc::channel();
    // let mut cache_watcher = notify::recommended_watcher(cache_tx)?;
    // cache_watcher.watch(&cache_path, notify::RecursiveMode::Recursive)?;

    fn monitor<F: 'static + FnMut() -> anyhow::Result<()> + Sync + Send>(
        path: &Path,
        mut f: F,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(path, notify::RecursiveMode::Recursive)?;

        Ok(std::thread::spawn(move || {
            for res in rx {
                if matches!(res?.kind, notify::EventKind::Modify(..)) {
                    f()?
                }
            }
            Ok(())
        }))
    }

    let source_monitor_thread = monitor(&path.clone(), move || {
        let scene: Scene = ron::de::from_reader(
            Command::new("cargo")
                .arg("run")
                .stdout(Stdio::piped())
                .current_dir(path.clone())
                .output()?
                .stdout
                .as_slice(),
        )?;
        let in_cache: HashMap<Worldline, BakedWorldline> = ron::de::from_reader(&cache_tempfile)?;
        let mut out_cache = HashMap::new();
        let baked_scene = scene.bake_with_cache(&in_cache, &mut out_cache);
        ron::ser::to_writer(&cache_tempfile, &out_cache)?;
        ron::ser::to_writer(&baked_scene_tempfile, &baked_scene)?;
        Ok(())
    })?;
    let baked_scene_monitor_thread = monitor(&baked_scene_path.clone(), move || {
        let baked_scene: BakedScene = ron::de::from_reader(&File::open(baked_scene_path.clone())?)?;
        baked_scene.render()?;
        Ok(())
    })?;

    let _ = source_monitor_thread.join().unwrap();
    let _ = baked_scene_monitor_thread.join().unwrap();

    // std::thread::spawn(|| {
    //     for res in source_watcher {

    //     }
    // })

    // let (tx, rx) = std::sync::mpsc::channel();
    // let mut watcher = notify::recommended_watcher(tx)?;
    // watcher.watch(&path, notify::RecursiveMode::Recursive)?;

    // let binary_path = src_path.join("../target/debug").canonicalize().unwrap();
    // let mut process: Option<Child> = None;
    // let mut cache = HashMap::new();
    // // start_child_process(&binary_path)?;

    // for res in rx {
    //     if matches!(res?.kind, notify::EventKind::Modify(..)) {
    //         // Recompile the child binary
    //         println!("Recompiling...");
    //         if let Some(process) = process.replace(
    //             Command::new(env!("CARGO_CRATE_NAME"))
    //                 .arg("internal-bake-incremental")
    //                 .arg("--path")
    //                 .arg(path)
    //                 .spawn()?,
    //         ) {
    //             if let Some(status) = process.try_wait()? {
    //                 if status.success() {}
    //             }
    //         }
    //         let path_clone = path.clone();
    //         //std::thread::spawn(move || fetch_scene(&src_path_clone));
    //     }
    // }
    Ok(())
}
