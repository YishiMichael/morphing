use std::io::BufRead;
use std::io::BufReader;
use std::panic::UnwindSafe;
use std::sync::Arc;
use std::sync::Mutex;

pub use inventory;
pub use morphing_macros::scene;

use super::config::Config;
use super::config::ConfigFallbackContent;
use super::config::ConfigValues;
use super::timeline::Supervisor;
use super::timeline::TimelineEntries;

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
    FilterFn(fn(&str) -> bool),
}

impl SceneFilter {
    fn is_match(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Whitelist(names) => names.contains(&name),
            Self::Blacklist(names) => !names.contains(&name),
            Self::FilterFn(f) => f(name),
        }
    }
}

fn serialize_and_write<T>(value: &T)
where
    T: serde::Serialize,
{
    println!("{}", ron::ser::to_string(&value).unwrap());
}

pub fn read_and_deserialize<R, T>(reader: &mut R) -> T
where
    R: BufRead,
    T: serde::de::DeserializeOwned,
{
    let mut buf = String::new();
    reader.read_line(&mut buf).unwrap();
    ron::de::from_str(&buf).unwrap()
}

pub fn export_scenes(scene_filter: SceneFilter) {
    // let mut buf = String::new();
    // std::io::stdin().read_line(&mut buf).unwrap();
    // let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
    // let mut override_settings_map = HashMap::new();

    let project_config_values = Mutex::new(ConfigValues::default());
    let project_redirected_result = ProjectRedirectedResult::execute(|| {
        for config_fallback_content in inventory::iter::<ConfigFallbackContent>() {
            project_config_values
                .lock()
                .unwrap()
                .overwrite(&config_fallback_content);
        }
        if let Ok(content) = std::fs::read_to_string("scene_config.toml") {
            project_config_values.lock().unwrap().overwrite(&content);
        } else {
            println!("Did not find project-level `scene_config.toml` file. Skipped.");
        }
    });
    serialize_and_write(&project_redirected_result);

    if project_redirected_result.is_ok() {
        let project_config_values = project_config_values.into_inner().unwrap();
        let handles: Vec<_> = inventory::iter::<SceneModule>()
            .map(|scene_module| {
                let name = scene_module.name.split_once("::").unwrap().1.to_string(); // Remove the crate root
                let scene_is_match = scene_filter.is_match(&name);
                let mut config_values = project_config_values.clone();
                std::thread::spawn(move || {
                    let scene_redirected_result = SceneRedirectedResult::execute(|| {
                        scene_is_match.then(|| {
                            if let Some(path) = scene_module.config_path {
                                let content = std::fs::read_to_string(path).unwrap();
                                config_values.overwrite(&content);
                            }
                            let config = Config::new(config_values.clone());
                            let supervisor = Supervisor::new(&config);
                            (scene_module.scene_fn)(&supervisor);
                            SceneData {
                                time: supervisor.time(),
                                timeline_entries: supervisor.into_timeline_entries(),
                                config_values,
                            }
                        })
                    });
                    serialize_and_write(&(name, scene_redirected_result));
                })
            })
            .collect();
        handles
            .into_iter()
            .for_each(|handle| handle.join().unwrap());
    }
}
