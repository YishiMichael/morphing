use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use morphing::toplevel::scene::SceneData;
use morphing::toplevel::settings::SceneSettings;

pub(crate) async fn open_project(path: PathBuf) -> anyhow::Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    let path = metadata
        .packages
        .iter()
        .map(|package| package.manifest_path.parent().unwrap())
        .find(|crate_path| path.starts_with(crate_path))
        .ok_or(anyhow::Error::msg("Failed to identify crate from manifest"))?
        .to_path_buf()
        .into_std_path_buf();
    Ok(path)
}

pub(crate) async fn compile_project(
    path: PathBuf,
    scene_settings: Arc<SceneSettings>,
) -> anyhow::Result<Vec<SceneData>> {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    writeln!(
        child.stdin.take().unwrap(),
        "{}",
        ron::ser::to_string(&*scene_settings)?
    )?;
    if !child.wait()?.success() {
        let mut stderr = BufReader::new(child.stderr.take().unwrap());
        let mut buf = String::new();
        stderr.read_to_string(&mut buf)?;
        Err(anyhow::Error::msg(buf))?;
    }
    let mut scenes = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
        let scene_data: SceneData = ron::de::from_str(&line?)?;
        scenes.push(scene_data);
    }
    Ok(scenes)
}
