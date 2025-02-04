use std::ops::RangeInclusive;
use std::time::Instant;

const PROGRESS_SPEED_EXPONENT_RANGE: RangeInclusive<f32> = -5.0..=5.0;
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
    speed_exponent: f32,
    play_direction: PlayDirection,
    playing: bool,
    anchor_instant: Instant,
}

#[derive(Clone, Debug)]
enum PlayDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub enum ProgressMessage {
    SetTime(f32),
    SetSpeedExponent(f32),
    SetPlayDirection(PlayDirection),
    SetPlaying(bool),
}

impl Progress {
    pub(crate) fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..=full_time,
            time: 0.0,
            speed_exponent: 0.0,
            play_direction: PlayDirection::Forward,
            playing: false,
            anchor_instant: Instant::now(),
        }
    }

    pub(crate) fn get_time(&self) -> f32 {
        self.time
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.playing
    }

    pub(crate) fn refresh_anchor(&mut self) {
        if self.playing {
            self.time += match self.play_direction {
                PlayDirection::Forward => 1.0,
                PlayDirection::Backward => -1.0,
            } * 2.0f32.powf(self.speed_exponent)
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
            ProgressMessage::SetSpeedExponent(speed_exponent) => {
                self.speed_exponent = speed_exponent;
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
