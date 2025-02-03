use std::ops::Range;
use std::time::Instant;

#[derive(Clone, Debug)]
pub(crate) struct Progress {
    time_interval: Range<f32>,
    time: f32,
    speed_level: i32,
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
    SetSpeedLevel(i32),
    SetPlayDirection(PlayDirection),
    SetPlaying(bool),
}

impl Progress {
    pub(crate) fn new(full_time: f32) -> Self {
        Self {
            time_interval: 0.0..full_time,
            time: 0.0,
            speed_level: 0,
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

    pub(crate) fn refresh_time(&mut self) {
        if self.playing {
            let mut time = self.time
                + match self.play_direction {
                    PlayDirection::Forward => 1.0,
                    PlayDirection::Backward => -1.0,
                } * 2.0f32.powi(self.speed_level)
                    * self.anchor_instant.elapsed().as_secs_f32();
            if !self.time_interval.contains(&time) {
                time = time.clamp(self.time_interval.start, self.time_interval.end);
                self.playing = false;
            }
            self.time = time;
            self.anchor_instant = Instant::now();
        }
    }

    pub(crate) fn update(&mut self, message: ProgressMessage) -> iced::Task<ProgressMessage> {
        self.refresh_time();
        match message {
            ProgressMessage::SetTime(time) => {
                self.time = time;
            }
            ProgressMessage::SetSpeedLevel(speed_level) => {
                self.speed_level = speed_level;
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
