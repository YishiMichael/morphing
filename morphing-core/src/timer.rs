pub type Time = f32;

#[derive(Default)]
pub struct Timer {
    time: Time,
}

impl Timer {
    pub fn wait(&mut self, time: Time) {
        self.time += time;
    }
}
