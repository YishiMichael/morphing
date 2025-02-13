use super::message::AppMessage;
use super::message::ProjectStateMessage;
use super::message::ProjectSuccessStateMessage;
use super::message::SceneStateMessage;
use super::message::SceneSuccessStateMessage;
use super::state::AppState;

pub(crate) fn theme(_state: &AppState) -> iced::Theme {
    iced::Theme::Dark
}

pub(crate) fn view(state: &AppState) -> iced::Element<AppMessage> {
    iced::widget::Column::new().push(menu_bar(state)).into()

    // iced::widget::Shader::new(self).into()
}

fn menu_bar(state: &AppState) -> iced::Element<AppMessage> {
    fn menu_bar_style(
        theme: &iced::Theme,
        _status: iced_aw::style::Status,
    ) -> iced_aw::widget::menu::Style {
        let palette = theme.extended_palette();
        iced_aw::widget::menu::Style {
            bar_background: palette.background.base.color.into(),
            bar_border: iced::Border::default(),
            bar_shadow: iced::Shadow::default(),
            bar_background_expand: iced::Padding::default(),
            menu_background: palette.background.base.color.into(),
            menu_border: iced::Border::default()
                .width(1.0)
                .color(palette.background.strong.color),
            menu_shadow: iced::Shadow::default(),
            menu_background_expand: iced::Padding::default(),
            path: iced::Color::default().into(),
            path_border: iced::Border::default(),
        }
        // match status {
        //     iced::widget::button::Status::Active => iced::widget::button::Style {
        //         background: None,
        //         text_color: pair.text,
        //         ..Default::default()
        //     },
        //     iced::widget::button::Status::Hovered
        //     | iced::widget::button::Status::Pressed => iced::widget::button::Style {
        //         background: Some(iced::Background::Color(pair.color)),
        //         text_color: pair.text,
        //         ..Default::default()
        //     },
        //     iced::widget::button::Status::Disabled => iced::widget::button::Style {
        //         background: None,
        //         text_color: pair.text.scale_alpha(0.3),
        //         ..Default::default()
        //     },
        // }
    }

    fn menu_button_style(
        theme: &iced::Theme,
        status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        let palette = theme.extended_palette();
        match status {
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: None,
                text_color: palette.secondary.base.text,
                ..Default::default()
            },
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(palette.secondary.weak.color)),
                    text_color: palette.secondary.base.text,
                    ..Default::default()
                }
            }
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: None,
                text_color: palette.secondary.base.text.scale_alpha(0.3),
                ..Default::default()
            },
        }
    }

    fn menu_button(text: &str) -> iced::widget::Button<AppMessage> {
        iced::widget::button(iced::widget::text(text).size(14.0))
            .padding([1.0, 6.0])
            .style(menu_button_style)
    }

    let open_message = Some(AppMessage::Open);
    let close_message = state
        .projects
        .get_active()
        .map(|project_state| AppMessage::Close(project_state.path.clone()));
    let save_video_message = state.projects.get_active().and_then(|project_state| {
        project_state
            .project_success_state
            .as_ref()
            .and_then(|project_success_state| {
                project_success_state
                    .scenes
                    .get_active()
                    .and_then(|scene_state| {
                        scene_state.scene_success_state.is_some().then_some(
                            AppMessage::ProjectState(
                                project_state.path.clone(),
                                ProjectStateMessage::ProjectSuccessState(
                                    ProjectSuccessStateMessage::SceneState(
                                        scene_state.name.clone(),
                                        SceneStateMessage::SceneSuccessState(
                                            SceneSuccessStateMessage::SaveVideo,
                                        ),
                                    ),
                                ),
                            ),
                        )
                    })
            })
    });
    let save_image_message = state.projects.get_active().and_then(|project_state| {
        project_state
            .project_success_state
            .as_ref()
            .and_then(|project_success_state| {
                project_success_state
                    .scenes
                    .get_active()
                    .and_then(|scene_state| {
                        scene_state.scene_success_state.is_some().then_some(
                            AppMessage::ProjectState(
                                project_state.path.clone(),
                                ProjectStateMessage::ProjectSuccessState(
                                    ProjectSuccessStateMessage::SceneState(
                                        scene_state.name.clone(),
                                        SceneStateMessage::SceneSuccessState(
                                            SceneSuccessStateMessage::SaveVideo,
                                        ),
                                    ),
                                ),
                            ),
                        )
                    })
            })
    });

    iced_aw::menu::MenuBar::new(Vec::from([
        iced_aw::menu::Item::with_menu(
            menu_button("File").on_press(AppMessage::Menu),
            iced_aw::menu::Menu::new(Vec::from([
                iced_aw::menu::Item::new(
                    menu_button("Open")
                        .width(iced::Length::Fill)
                        .on_press_maybe(open_message),
                ),
                iced_aw::menu::Item::new(
                    menu_button("Close")
                        .width(iced::Length::Fill)
                        .on_press_maybe(close_message),
                ),
                iced_aw::menu::Item::new(
                    menu_button("Save Video")
                        .width(iced::Length::Fill)
                        .on_press_maybe(save_video_message),
                ),
                iced_aw::menu::Item::new(
                    menu_button("Save Image")
                        .width(iced::Length::Fill)
                        .on_press_maybe(save_image_message),
                ),
            ]))
            .width(180.0)
            .offset(2.0),
        ),
        iced_aw::menu::Item::with_menu(
            menu_button("Setting").on_press(AppMessage::Menu),
            iced_aw::menu::Menu::new(Vec::from([
                iced_aw::menu::Item::with_menu(
                    menu_button("Default Scene Settings")
                        .width(iced::Length::Fill)
                        .on_press(AppMessage::Menu),
                    iced_aw::menu::Menu::new(Vec::from([
                        iced_aw::menu::Item::new(
                            menu_button("Open")
                                .width(iced::Length::Fill)
                                .on_press(AppMessage::Menu),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Close")
                                .width(iced::Length::Fill)
                                .on_press(AppMessage::Menu),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Save Video")
                                .width(iced::Length::Fill)
                                .on_press(AppMessage::Menu),
                        ),
                        iced_aw::menu::Item::new(
                            menu_button("Save Image")
                                .width(iced::Length::Fill)
                                .on_press(AppMessage::Menu),
                        ),
                    ]))
                    .width(180.0)
                    .offset(2.0),
                ),
                iced_aw::menu::Item::new(menu_button("Video Settings").width(iced::Length::Fill)),
            ]))
            .width(180.0)
            .offset(2.0),
        ),
    ]))
    .style(menu_bar_style)
    .into()
}
