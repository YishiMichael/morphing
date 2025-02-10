use std::ops::Add;
use std::ops::Mul;
use std::ops::RangeInclusive;
use std::time::Instant;

const PROGRESS_SPEED_RANGE: RangeInclusive<f32> = 0.03125..=32.0;
const PLAY_PAUSE_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::Space);
const FAST_FORWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight);
const FAST_BACKWARD_KEY: iced::keyboard::Key =
    iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft);
const FAST_SKIP_SECONDS: f32 = 5.0;

#[derive(Clone, Debug)]
pub(crate) struct Progress {
    time_interval: RangeInclusive<f32>,
    time: f32,
    speed: f32,
    play_direction: PlayDirection,
    playing: bool,
    anchor_instant: Instant,
}

#[derive(Clone, Debug)]
enum PlayDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug)]
pub enum ProgressMessage {
    SetTime(f32),
    SetSpeed(f32),
    SetPlayDirection(PlayDirection),
    SetPlaying(bool),
}

impl Progress {
    pub(crate) fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..=full_time,
            time: 0.0,
            speed: 1.0,
            play_direction: PlayDirection::Forward,
            playing: false,
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

    pub(crate) fn update(&mut self, message: ProgressMessage) -> iced::Task<ProgressMessage> {
        self.refresh_anchor();
        match message {
            ProgressMessage::SetTime(time) => {
                self.time = time;
            }
            ProgressMessage::SetSpeed(speed) => {
                self.speed = speed;
            }
            ProgressMessage::SetPlayDirection(play_direction) => {
                self.play_direction = play_direction;
            }
            ProgressMessage::SetPlaying(playing) => {
                self.playing = playing;
            }
        }
        iced::Task::none()
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
