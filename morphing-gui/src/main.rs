mod app;

use app::AppState;

fn main() -> iced::Result {
    iced::application("Morphing Viewer", AppState::update, AppState::view).run()
}
