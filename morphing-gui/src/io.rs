use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use morphing::toplevel::scene::SceneData;
use morphing::toplevel::settings::SceneSettings;

// pub(crate) async fn open_project(path: PathBuf) -> anyhow::Result<PathBuf> {
//     let metadata = cargo_metadata::MetadataCommand::new().exec()?;
//     let path = metadata
//         .packages
//         .iter()
//         .map(|package| package.manifest_path.parent().unwrap())
//         .find(|crate_path| path.starts_with(crate_path))
//         .ok_or(anyhow::Error::msg("Failed to identify crate from manifest"))?
//         .to_path_buf()
//         .into_std_path_buf();
//     Ok(path)
// }

pub(crate) async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(PathBuf::from)
}

pub(crate) async fn pick_folders() -> Option<Vec<PathBuf>> {
    rfd::AsyncFileDialog::new()
        .pick_folders()
        .await
        .map(|folders| folders.into_iter().map(PathBuf::from).collect())
}

pub(crate) async fn pick_save_file(name: &str, extensions: &[&str]) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(name, extensions)
        .save_file()
        .await
        .map(PathBuf::from)
}

pub(crate) async fn compile_project(
    path: PathBuf,
    scene_settings: Arc<SceneSettings>,
) -> Result<Vec<(String, SceneData)>, String> {

    async fn compile_project_inner(
        path: PathBuf,
        scene_settings: &SceneSettings,
    ) -> anyhow::Result<Vec<(String, SceneData)>> {
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
            ron::ser::to_string(&scene_settings)?
        )?;
        if !child.wait()?.success() {
            let mut stderr = BufReader::new(child.stderr.take().unwrap());
            let mut buf = String::new();
            stderr.read_to_string(&mut buf)?;
            Err(anyhow::Error::msg(buf))?;
        }
        let mut scenes_data = Vec::new();
        for line in BufReader::new(child.stdout.take().unwrap()).lines() {
            let (name, scene_data): (String, SceneData) = ron::de::from_str(&line?)?;
            scenes_data.push((name, scene_data));
        }
        Ok(scenes_data)
    }

    compile_project_inner(path, &scene_settings).await.map_err(|error| error.to_string())
}
