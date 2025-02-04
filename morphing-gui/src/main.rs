mod app;
mod collection;
mod io;
mod logger;
mod progress;

use app::AppState;

fn main() -> iced::Result {
    iced::application("Morphing GUI", AppState::update, AppState::view).run()
}
