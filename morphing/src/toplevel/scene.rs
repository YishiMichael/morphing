use std::io::BufRead;
use std::io::BufReader;
use std::panic::UnwindSafe;
use std::sync::Arc;
use std::sync::Mutex;

pub use inventory;
pub use morphing_macros::scene;

use super::super::timelines::timeline::Supervisor;
use super::super::timelines::timeline::TimelineEntries;
use super::config::Config;
use super::config::ConfigValues;
// use super::settings::SceneSettings;
// use super::settings::VideoSettings;
// use super::world::World;

pub struct SceneModule {
    pub name: &'static str,
    pub config_path: Option<&'static str>,
    pub scene_fn: fn(&Supervisor),
}

inventory::collect!(SceneModule);

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RedirectedResult<R> {
    pub result: Result<R, ()>,
    pub stdout_lines: Arc<Vec<String>>,
    pub stderr_lines: Arc<Vec<String>>,
}

impl<R> RedirectedResult<R> {
    fn execute<F>(f: F) -> Self
    where
        F: FnOnce() -> R + UnwindSafe,
    {
        let shh_stdout = BufReader::new(shh::stdout().unwrap());
        let shh_stderr = BufReader::new(shh::stderr().unwrap());
        let result = std::panic::catch_unwind(f).map_err(|_| ());
        Self {
            result,
            stdout_lines: Arc::new(shh_stdout.lines().map(Result::unwrap).collect()),
            stderr_lines: Arc::new(shh_stderr.lines().map(Result::unwrap).collect()),
        }
    }

    fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
}

pub type ProjectRedirectedResult = RedirectedResult<()>;

pub type SceneRedirectedResult = RedirectedResult<Option<SceneData>>;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SceneData {
    pub time: f32,
    pub timeline_entries: TimelineEntries,
    pub config_values: ConfigValues,
}

pub enum SceneFilter {
    All,
    Whitelist(&'static [&'static str]),
    Blacklist(&'static [&'static str]),
    Regex(lazy_regex::Lazy<lazy_regex::Regex>),
    FilterFn(fn(&str) -> bool),
}

impl SceneFilter {
    fn is_match(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Whitelist(names) => names.contains(&name),
            Self::Blacklist(names) => !names.contains(&name),
            Self::Regex(regex) => regex.is_match(name),
            Self::FilterFn(f) => f(name),
        }
    }
}

pub fn export_scenes(scene_filter: SceneFilter) {
    // let mut buf = String::new();
    // std::io::stdin().read_line(&mut buf).unwrap();
    // let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
    // let mut override_settings_map = HashMap::new();

    let project_config_values = Mutex::new(ConfigValues::default());
    let project_redirected_result = ProjectRedirectedResult::execute(|| {
        {
            let content =
                std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/scene_config.toml"))
                    .unwrap();
            project_config_values.lock().unwrap().overwrite(&content);
        }
        if let Ok(content) = std::fs::read_to_string("scene_config.toml") {
            project_config_values.lock().unwrap().overwrite(&content);
        } else {
            println!("Did not find project-level `scene_config.toml` file. Skipped.");
        }
    });
    println!(
        "{}",
        ron::ser::to_string(&project_redirected_result).unwrap()
    );

    if project_redirected_result.is_ok() {
        for scene_module in inventory::iter::<SceneModule>() {
            let name = scene_module.name.split_once("::").unwrap().1.to_string(); // Remove the crate root
            let scene_redirected_result = SceneRedirectedResult::execute(|| {
                let mut config_values = project_config_values.lock().unwrap().clone();
                if let Some(path) = scene_module.config_path {
                    let content = std::fs::read_to_string(path).unwrap();
                    config_values.overwrite(&content);
                }
                scene_filter.is_match(&name).then(|| {
                    Supervisor::visit(
                        &Config::new(config_values.clone()),
                        scene_module.scene_fn,
                        |time, timeline_entries, _| SceneData {
                            time,
                            timeline_entries,
                            config_values,
                        },
                    )
                })
            });
            println!(
                "{}",
                ron::ser::to_string(&(name, scene_redirected_result)).unwrap()
            );
        }
    }
}
