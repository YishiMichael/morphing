use std::ops::Add;
use std::ops::Mul;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::slice::Iter;
use std::time::Instant;
use std::time::SystemTime;

use morphing_core::config::Config;
use morphing_core::timeline::TimelineEntries;

const PROGRESS_SPEED_RANGE: RangeInclusive<f32> = 0.03125..=32.0;
const PLAY_PAUSE_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::Space);
const FAST_FORWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight);
const FAST_BACKWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft);
const FAST_SKIP_SECONDS: f32 = 5.0;

#[derive(Debug, Default)]
pub(crate) struct AppState {
    pub(crate) projects: Collection<ProjectState>,
}

#[derive(Debug)]
pub(crate) struct ProjectState {
    pub(crate) path: PathBuf,
    pub(crate) watching: bool, // TODO
    pub(crate) project_success_state: Option<ProjectSuccessState>,
    pub(crate) logger: Logger,
    pub(crate) generation: usize,
}

#[derive(Debug, Default)]
pub(crate) struct ProjectSuccessState {
    pub(crate) scenes: Collection<SceneState>,
}

#[derive(Debug)]
pub(crate) struct SceneState {
    pub(crate) name: String,
    pub(crate) scene_success_state: Option<SceneSuccessState>,
    pub(crate) logger: Logger,
    pub(crate) generation: usize,
}

#[derive(Debug, Default)]
pub(crate) struct SceneSuccessState {
    pub(crate) progress: Progress,
    pub(crate) timeline_entries: TimelineEntries,
    pub(crate) config: Config,
}

#[derive(Debug)]
pub(crate) struct Collection<T> {
    items: Vec<T>,
    active_index: Option<usize>,
}

impl<T> Default for Collection<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            active_index: None,
        }
    }
}

impl<T> FromIterator<T> for Collection<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: iter.into_iter().collect(),
            active_index: None,
        }
    }
}

pub(crate) trait CollectionItem {
    type Key: PartialEq;

    fn key(&self) -> &Self::Key;
}

impl<T, K> Collection<T>
where
    T: CollectionItem<Key = K>,
    K: PartialEq,
{
    pub(crate) fn get_active(&self) -> Option<&T> {
        self.active_index
            .map(|active_index| self.items.get(active_index))
            .flatten()
    }

    pub(crate) fn set_active(&mut self, key: Option<&K>) {
        self.active_index = key
            .map(|key| self.items.iter().position(|item| item.key() == key))
            .flatten();
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        self.items.iter()
    }

    pub(crate) fn active_find_or_insert_with<F>(&mut self, key: K, f: F) -> &mut T
    where
        F: FnOnce(K) -> T,
    {
        let index = self
            .items
            .iter_mut()
            .position(|item| item.key() == &key)
            .unwrap_or_else(|| {
                let index = self
                    .active_index
                    .map(|index| index + 1)
                    .unwrap_or(self.items.len());
                self.items.insert(index, f(key));
                index
            });
        self.active_index = Some(index);
        self.items.get_mut(index).unwrap()
    }

    pub(crate) fn inactive_find_or_insert_with<F>(&mut self, key: K, f: F) -> &mut T
    where
        F: FnOnce(K) -> T,
    {
        let index = self
            .items
            .iter_mut()
            .position(|item| item.key() == &key)
            .unwrap_or_else(|| {
                let index = self.items.len();
                self.items.insert(index, f(key));
                index
            });
        self.items.get_mut(index).unwrap()
    }

    pub(crate) fn remove(&mut self, key: &K) {
        if let Some(index) = self.items.iter().position(|item| item.key() == key) {
            self.items.remove(index);
            if self.active_index == Some(index) {
                let index = index.saturating_sub(1);
                self.active_index = (index < self.items.len()).then_some(index);
            }
        }
    }
}

impl CollectionItem for ProjectState {
    type Key = PathBuf;

    fn key(&self) -> &Self::Key {
        &self.path
    }
}

impl CollectionItem for SceneState {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }
}

#[derive(Debug)]
struct LogRecord {
    timestamp: SystemTime,
    level: LogLevel,
    message: String,
}

// https://docs.rs/log/latest/src/log/lib.rs.html#484-508
#[derive(Debug)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    // https://docs.rs/env_logger/latest/src/env_logger/fmt/mod.rs.html#159
    fn color(&self) -> iced::Color {
        match self {
            Self::Error => iced::Color::from_rgb8(0xFF, 0x55, 0x55), // Red
            Self::Warn => iced::Color::from_rgb8(0xFF, 0xFF, 0x55),  // Yellow
            Self::Info => iced::Color::from_rgb8(0x55, 0xFF, 0x55),  // Green
            Self::Debug => iced::Color::from_rgb8(0x55, 0x55, 0xFF), // Blue
            Self::Trace => iced::Color::from_rgb8(0x55, 0xFF, 0xFF), // Cyan
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Logger(Vec<LogRecord>);

impl Logger {
    pub(crate) fn log<S>(&mut self, level: LogLevel, message: S)
    where
        S: AsRef<str>,
    {
        self.0.push(LogRecord {
            timestamp: SystemTime::now(),
            level,
            message: message.as_ref().to_string(),
        });
    }
}

// Use humantime::format_rfc3339_seconds
// [2025-01-01T12:34:56Z WARN ] message

#[derive(Clone, Debug)]
pub(crate) struct Progress {
    pub(crate) time: f32,
    pub(crate) speed: f32,
    pub(crate) play_direction: PlayDirection,
    pub(crate) playing: bool,
    time_interval: RangeInclusive<f32>,
    anchor_instant: Instant,
}

#[derive(Clone, Debug)]
pub(crate) enum PlayDirection {
    Forward,
    Backward,
}

impl Progress {
    pub(crate) fn new(full_time: f32) -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            play_direction: PlayDirection::Forward,
            playing: false,
            time_interval: 0.0..=full_time,
            anchor_instant: Instant::now(),
        }
    }

    pub(crate) fn time(&self) -> f32 {
        self.time
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.playing
    }

    pub(crate) fn frame_range(&self, fps: f32) -> RangeStepInclusive<f32> {
        RangeStepInclusive {
            range: self.time_interval.clone(),
            start: *match self.play_direction {
                PlayDirection::Forward => self.time_interval.start(),
                PlayDirection::Backward => self.time_interval.end(),
            },
            step: match self.play_direction {
                PlayDirection::Forward => 1.0,
                PlayDirection::Backward => -1.0,
            } * self.speed
                / fps,
            count: 0,
        }
    }

    pub(crate) fn refresh_anchor(&mut self) {
        if self.playing {
            self.time += match self.play_direction {
                PlayDirection::Forward => 1.0,
                PlayDirection::Backward => -1.0,
            } * self.speed
                * self.anchor_instant.elapsed().as_secs_f32();
            if !self.time_interval.contains(&self.time) {
                self.time = self
                    .time
                    .clamp(*self.time_interval.start(), *self.time_interval.end());
                self.playing = false;
            }
            self.anchor_instant = Instant::now();
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new(0.0)
    }
}

pub(crate) struct RangeStepInclusive<T> {
    range: RangeInclusive<T>,
    start: T,
    step: T,
    count: usize,
}

impl<T> Iterator for RangeStepInclusive<T>
where
    T: Copy + From<usize> + PartialOrd + Add<Output = T> + Mul<Output = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.start + self.step * T::from(self.count);
        if self.range.contains(&item) {
            self.count += 1;
            Some(item)
        } else {
            None
        }
    }
}
