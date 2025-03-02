// use std::io::BufRead;
// use std::io::BufReader;
// use std::panic::UnwindSafe;
// use std::sync::Arc;
// use std::sync::Mutex;

// pub use inventory;
// pub use morphing_macros::scene;

// use super::alive::AliveRoot;
// use super::config::Config;
// use super::config::ConfigFallbackContent;
// use super::config::ConfigValues;

// pub struct SceneModule {
//     pub name: &'static str,
//     pub config_content: Option<&'static str>,
//     pub scene_fn: fn(&AliveRoot),
// }

// inventory::collect!(SceneModule);

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct RedirectedOutput<R> {
//     pub result: Result<R, ()>,
//     pub stdout_lines: Arc<Vec<String>>,
//     pub stderr_lines: Arc<Vec<String>>,
// }

// impl<R> RedirectedOutput<R> {
//     fn execute<F>(f: F) -> Self
//     where
//         F: FnOnce() -> R + UnwindSafe,
//     {
//         let shh_stdout = BufReader::new(shh::stdout().unwrap());
//         let shh_stderr = BufReader::new(shh::stderr().unwrap());
//         let result = std::panic::catch_unwind(f).map_err(|_| {});
//         Self {
//             result,
//             stdout_lines: Arc::new(shh_stdout.lines().map(Result::unwrap).collect()),
//             stderr_lines: Arc::new(shh_stderr.lines().map(Result::unwrap).collect()),
//         }
//     }

//     fn is_ok(&self) -> bool {
//         self.result.is_ok()
//     }
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub enum LineOutput {
//     Project(RedirectedOutput<()>),
//     Scene(String, RedirectedOutput<Option<SceneData>>),
// }

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct SceneData {
//     pub time: f32,
//     pub timeline_entries: TimelineEntries,
//     pub config_values: ConfigValues,
// }

// pub enum SceneFilter {
//     All,
//     Whitelist(&'static [&'static str]),
//     Blacklist(&'static [&'static str]),
//     FilterFn(fn(&str) -> bool),
// }

// impl SceneFilter {
//     fn is_match(&self, name: &str) -> bool {
//         match self {
//             Self::All => true,
//             Self::Whitelist(names) => names.contains(&name),
//             Self::Blacklist(names) => !names.contains(&name),
//             Self::FilterFn(f) => f(name),
//         }
//     }
// }

// fn serialize_and_write(value: &LineOutput) -> Result<(), ron::Error> {
//     println!("{}", ron::ser::to_string(&value)?);
//     Ok(())
// }

// pub fn read_and_deserialize(s: &str) -> Result<LineOutput, ron::error::SpannedError> {
//     ron::de::from_str(s)
// }

// pub fn export_scenes(scene_filter: SceneFilter, config_content: Option<&'static str>) {
//     // let mut buf = String::new();
//     // std::io::stdin().read_line(&mut buf).unwrap();
//     // let scene_settings: SceneSettings = ron::de::from_str(&buf).unwrap();
//     // let mut override_settings_map = HashMap::new();

//     let project_config_values = Mutex::new(ConfigValues::default());
//     let project_redirected_result = RedirectedOutput::execute(|| {
//         for config_fallback_content in inventory::iter::<ConfigFallbackContent>() {
//             project_config_values
//                 .lock()
//                 .unwrap()
//                 .overwrite(&config_fallback_content);
//         }
//         if let Some(config_content) = config_content {
//             project_config_values
//                 .lock()
//                 .unwrap()
//                 .overwrite(&config_content);
//         }
//     });
//     let is_ok = project_redirected_result.is_ok();
//     serialize_and_write(&LineOutput::Project(project_redirected_result)).unwrap();

//     if is_ok {
//         let project_config_values = project_config_values.into_inner().unwrap();
//         let handles: Vec<_> = inventory::iter::<SceneModule>()
//             .map(|scene_module| {
//                 let name = scene_module.name.split_once("::").unwrap().1.to_string(); // Remove the crate root
//                 let scene_is_match = scene_filter.is_match(&name);
//                 let mut config_values = project_config_values.clone();
//                 std::thread::spawn(move || {
//                     let scene_redirected_result = RedirectedOutput::execute(|| {
//                         scene_is_match.then(|| {
//                             if let Some(config_content) = scene_module.config_content {
//                                 config_values.overwrite(&config_content);
//                             }
//                             let config = Config::new(config_values.clone());
//                             let supervisor = Supervisor::new(&config);
//                             (scene_module.scene_fn)(&supervisor);
//                             SceneData {
//                                 time: supervisor.time(),
//                                 timeline_entries: supervisor.into_timeline_entries(),
//                                 config_values,
//                             }
//                         })
//                     });
//                     serialize_and_write(&LineOutput::Scene(name, scene_redirected_result)).unwrap();
//                 })
//             })
//             .collect();
//         handles
//             .into_iter()
//             .for_each(|handle| handle.join().unwrap());
//     }
// }
