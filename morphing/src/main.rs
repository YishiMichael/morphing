use morphing::toplevel::app::app::State;

fn main() -> iced::Result {
    iced::application("morphing", State::update, State::view).run()
}
