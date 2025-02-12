use std::io::BufRead;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::ChildStdout;
use std::task::Context;
use std::task::Poll;

use morphing::toplevel::scene::ProjectRedirectedResult;
use morphing::toplevel::scene::SceneRedirectedResult;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;

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
) -> std::io::Result<
    Result<
        (
            ProjectRedirectedResult,
            SceneStream<
                tokio_stream::wrappers::LinesStream<
                    tokio::io::BufReader<tokio::process::ChildStdout>,
                >,
            >,
        ),
        String,
    >,
> {
    let mut child = std::process::Command::new("cargo")
        .arg("run")
        .current_dir(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    Ok(if child.wait()?.success() {
        ChildStdout
        let mut lines = std::io::BufReader::new(child.stdout.take().unwrap()).lines();
        let line = lines.next()?.unwrap();
        Ok((
            ron::de::from_str(&line).unwrap(),
            SceneStream(tokio_stream::wrappers::LinesStream::new(lines)),
        ))
    } else {
        let mut buf = String::new();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut buf)
            .await?;
        Err(buf)
    })
}

pub(crate) struct SceneStream<S>(S);

impl<S> futures_core::Stream for SceneStream<S>
where
    S: Unpin + futures_core::Stream<Item = std::io::Result<String>>,
{
    type Item = (String, SceneRedirectedResult);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx).map(|line| {
            line.map(|line| {
                let line = line.unwrap();
                ron::de::from_str(&line).unwrap()
            })
        })
    }
}
