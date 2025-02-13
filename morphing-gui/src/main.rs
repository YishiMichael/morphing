#![feature(option_get_or_insert_default)]

mod message;
mod state;
mod update;
mod view;

fn main() -> iced::Result {
    iced::application("Morphing GUI", update::update, view::view)
        .theme(view::theme)
        .run()
}
