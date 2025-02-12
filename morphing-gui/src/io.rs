use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::path::PathBuf;
use std::process::ChildStdout;

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

pub(crate) async fn pick_save_file(
    filter_name: &str,
    filter_extensions: &[&str],
) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(filter_name, filter_extensions)
        .save_file()
        .await
        .map(PathBuf::from)
}

pub(crate) async fn compile_project(
    path: PathBuf,
) -> std::io::Result<futures::stream::Iter<Lines<BufReader<ChildStdout>>>> {
    let mut child = std::process::Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    Ok(futures::stream::iter(BufReader::new(child.stdout.take().unwrap()).lines()))
}
