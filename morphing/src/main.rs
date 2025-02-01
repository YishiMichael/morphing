use morphing::toplevel::app::app::State;

fn main() -> iced::Result {
    iced::application("Morphing Viewer", State::update, State::view).run()
}
