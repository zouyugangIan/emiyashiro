//! 设置界面（主菜单 / 暂停菜单共用的音频等选项）

use crate::resources::{AudioSettings, GameAssets};
use bevy::prelude::*;

/// 设置浮层根节点（覆盖在当前界面上方）
#[derive(Component)]
pub struct SettingsOverlayRoot;

#[derive(Component)]
pub struct SettingsBackButton;

#[derive(Component)]
pub struct MenuSettingsButton;

#[derive(Component)]
pub struct PauseSettingsButton;

#[derive(Component)]
pub struct SettingsPanelTitle;

#[derive(Component)]
pub struct VolumeIconButton;

#[derive(Component)]
pub struct VolumeIconImage;

#[derive(Component)]
pub struct VolumeDownButton;

#[derive(Component)]
pub struct VolumeUpButton;

#[derive(Component)]
pub struct VolumeBarFill;

#[derive(Component)]
pub struct VolumePercentText;

#[derive(Resource)]
pub struct VolumeControlState {
    pub last_audible_volume: f32,
}

impl Default for VolumeControlState {
    fn default() -> Self {
        Self {
            last_audible_volume: 0.5,
        }
    }
}

type VolumeControlInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static VolumeIconButton>,
        Option<&'static VolumeDownButton>,
        Option<&'static VolumeUpButton>,
    ),
    (
        Changed<Interaction>,
        With<Button>,
        Or<(
            With<VolumeIconButton>,
            With<VolumeDownButton>,
            With<VolumeUpButton>,
        )>,
    ),
>;

mod palette {
    use bevy::prelude::Color;

    pub const OVERLAY: Color = Color::srgba(0.02, 0.02, 0.04, 0.72);
    pub const PANEL: Color = Color::srgba(0.07, 0.075, 0.09, 0.97);
    pub const PANEL_BORDER: Color = Color::srgba(0.95, 0.74, 0.42, 0.36);
    pub const SECTION: Color = Color::srgba(0.10, 0.105, 0.13, 0.86);
    pub const SECTION_BORDER: Color = Color::srgba(0.50, 0.54, 0.62, 0.28);
    pub const TITLE: Color = Color::srgba(0.95, 0.92, 0.86, 1.0);
    pub const LABEL: Color = Color::srgba(0.78, 0.78, 0.84, 0.96);
    pub const MUTED_LABEL: Color = Color::srgba(0.58, 0.60, 0.67, 0.92);
    pub const BTN_IDLE: Color = Color::srgba(0.13, 0.14, 0.18, 0.94);
    pub const BTN_PRESS: Color = Color::srgba(0.35, 0.24, 0.16, 0.98);
    pub const BTN_BORDER_IDLE: Color = Color::srgba(0.72, 0.58, 0.36, 0.32);
    pub const TRACK: Color = Color::srgba(0.08, 0.09, 0.12, 0.95);
    pub const TRACK_BORDER: Color = Color::srgba(0.48, 0.50, 0.58, 0.30);
    pub const FILL: Color = Color::srgba(0.95, 0.63, 0.24, 0.96);
    pub const PERCENT: Color = Color::srgba(0.91, 0.90, 0.94, 0.96);
    pub const BACK_IDLE: Color = Color::srgba(0.11, 0.12, 0.15, 0.92);
    pub const BACK_HOVER: Color = Color::srgba(0.22, 0.18, 0.13, 0.96);
}

pub fn open_settings_overlay_from_resources(
    commands: &mut Commands,
    game_assets: Option<&GameAssets>,
    audio_settings: Option<&AudioSettings>,
    existing: &Query<(), With<SettingsOverlayRoot>>,
) {
    let font = game_assets
        .map(|assets| assets.font.clone())
        .unwrap_or_default();
    let volume_icon = game_assets
        .map(|assets| assets.volume_icon.clone())
        .unwrap_or_default();
    let volume_muted_icon = game_assets
        .map(|assets| assets.volume_muted_icon.clone())
        .unwrap_or_default();
    let initial_volume = audio_settings
        .map(|settings| settings.master_volume)
        .unwrap_or(1.0);

    open_settings_overlay(
        commands,
        font,
        volume_icon,
        volume_muted_icon,
        initial_volume,
        existing,
    );
}

pub fn open_settings_overlay(
    commands: &mut Commands,
    font: Handle<Font>,
    volume_icon: Handle<Image>,
    volume_muted_icon: Handle<Image>,
    initial_volume: f32,
    existing: &Query<(), With<SettingsOverlayRoot>>,
) {
    if !existing.is_empty() {
        return;
    }

    let initial_volume = initial_volume.clamp(0.0, 1.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(palette::OVERLAY),
            ZIndex(20),
            SettingsOverlayRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(90.0),
                    min_width: Val::Px(320.0),
                    max_width: Val::Px(460.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::axes(Val::Px(30.0), Val::Px(26.0)),
                    row_gap: Val::Px(20.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(palette::PANEL),
                BorderColor::all(palette::PANEL_BORDER),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(crate::systems::text_constants::SettingsMenuText::TITLE),
                    TextFont {
                        font: font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(palette::TITLE),
                    SettingsPanelTitle,
                ));

                panel.spawn((
                    Text::new(crate::systems::text_constants::SettingsMenuText::AUDIO_SECTION),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(palette::MUTED_LABEL),
                ));

                spawn_master_volume_control(
                    panel,
                    font.clone(),
                    volume_icon,
                    volume_muted_icon,
                    initial_volume,
                );

                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(44.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(palette::BACK_IDLE),
                        BorderColor::all(palette::BTN_BORDER_IDLE),
                        SettingsBackButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(crate::systems::text_constants::SettingsMenuText::BACK),
                            TextFont {
                                font,
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(palette::TITLE),
                        ));
                    });
            });
        });
}

fn despawn_settings_overlays(commands: &mut Commands, overlays: impl IntoIterator<Item = Entity>) {
    for entity in overlays {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_settings_overlay(
    mut commands: Commands,
    overlay_query: Query<Entity, With<SettingsOverlayRoot>>,
) {
    despawn_settings_overlays(&mut commands, overlay_query.iter());
}

pub fn close_settings_overlay_on_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    overlay_query: Query<Entity, With<SettingsOverlayRoot>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        despawn_settings_overlays(&mut commands, overlay_query.iter());
    }
}

type SettingsBackInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<SettingsBackButton>),
>;

pub fn handle_settings_back_button(
    mut interaction_query: SettingsBackInteractionQuery,
    mut commands: Commands,
    overlay_query: Query<Entity, With<SettingsOverlayRoot>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                despawn_settings_overlays(&mut commands, overlay_query.iter());
            }
            Interaction::Hovered => {
                *color = BackgroundColor(palette::BACK_HOVER);
            }
            Interaction::None => {
                *color = BackgroundColor(palette::BACK_IDLE);
            }
        }
    }
}

pub fn handle_volume_control_interactions(
    mut interaction_query: VolumeControlInteractionQuery,
    mut audio_settings: ResMut<AudioSettings>,
    mut volume_state: ResMut<VolumeControlState>,
) {
    const VOLUME_STEP: f32 = 0.1;

    for (interaction, mut color, icon_button, down_button, up_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if audio_settings.master_volume > 0.0 {
                    volume_state.last_audible_volume = audio_settings.master_volume.clamp(0.1, 1.0);
                }

                if icon_button.is_some() {
                    audio_settings.master_volume = if audio_settings.master_volume > 0.0 {
                        0.0
                    } else {
                        volume_state.last_audible_volume.clamp(VOLUME_STEP, 1.0)
                    };
                } else if down_button.is_some() {
                    audio_settings.master_volume =
                        (audio_settings.master_volume - VOLUME_STEP).clamp(0.0, 1.0);
                } else if up_button.is_some() {
                    audio_settings.master_volume =
                        (audio_settings.master_volume + VOLUME_STEP).clamp(0.0, 1.0);
                }

                audio_settings.master_volume = (audio_settings.master_volume * 10.0).round() / 10.0;
                if audio_settings.master_volume > 0.0 {
                    volume_state.last_audible_volume = audio_settings.master_volume;
                }
                *color = BackgroundColor(palette::BTN_PRESS);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(palette::BACK_HOVER);
            }
            Interaction::None => {
                *color = BackgroundColor(palette::BTN_IDLE);
            }
        }
    }
}

pub fn update_volume_control_display(
    audio_settings: Res<AudioSettings>,
    game_assets: Option<Res<GameAssets>>,
    mut fill_query: Query<&mut Node, With<VolumeBarFill>>,
    mut text_query: Query<&mut Text, With<VolumePercentText>>,
    mut icon_query: Query<&mut ImageNode, With<VolumeIconImage>>,
) {
    if !audio_settings.is_changed() {
        return;
    }

    let volume = audio_settings.master_volume.clamp(0.0, 1.0);
    let percent = volume * 100.0;

    for mut node in fill_query.iter_mut() {
        node.width = Val::Percent(percent);
    }

    for mut text in text_query.iter_mut() {
        **text = format!("{percent:.0}%");
    }

    let Some(assets) = game_assets else {
        return;
    };
    let icon = if volume <= 0.0 {
        assets.volume_muted_icon.clone()
    } else {
        assets.volume_icon.clone()
    };
    for mut image in icon_query.iter_mut() {
        image.image = icon.clone();
    }
}

fn spawn_master_volume_control(
    parent: &mut ChildSpawnerCommands<'_>,
    font: Handle<Font>,
    volume_icon: Handle<Image>,
    volume_muted_icon: Handle<Image>,
    initial_volume: f32,
) {
    let initial_icon = if initial_volume <= 0.0 {
        volume_muted_icon
    } else {
        volume_icon
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette::SECTION),
            BorderColor::all(palette::SECTION_BORDER),
        ))
        .with_children(|section| {
            section.spawn((
                Text::new(crate::systems::text_constants::SettingsMenuText::MASTER_VOLUME),
                TextFont {
                    font: font.clone(),
                    font_size: 17.0,
                    ..default()
                },
                TextColor(palette::LABEL),
            ));

            section
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                },))
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(palette::BTN_IDLE),
                        BorderColor::all(palette::BTN_BORDER_IDLE),
                        VolumeIconButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            ImageNode::new(initial_icon),
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                            VolumeIconImage,
                        ));
                    });

                    row.spawn((
                        Node {
                            flex_grow: 1.0,
                            min_width: Val::Px(112.0),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Stretch,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(palette::TRACK),
                        BorderColor::all(palette::TRACK_BORDER),
                    ))
                    .with_children(|track| {
                        track.spawn((
                            Node {
                                width: Val::Percent(initial_volume * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(palette::FILL),
                            VolumeBarFill,
                        ));
                    });

                    row.spawn((
                        Text::new(format!("{:.0}%", initial_volume * 100.0)),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(palette::PERCENT),
                        Node {
                            width: Val::Px(46.0),
                            ..default()
                        },
                        VolumePercentText,
                    ));

                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(palette::BTN_IDLE),
                        BorderColor::all(palette::BTN_BORDER_IDLE),
                        VolumeDownButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(
                                crate::systems::text_constants::SettingsMenuText::VOLUME_DOWN,
                            ),
                            TextFont {
                                font: font.clone(),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(palette::TITLE),
                        ));
                    });

                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(palette::BTN_IDLE),
                        BorderColor::all(palette::BTN_BORDER_IDLE),
                        VolumeUpButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(crate::systems::text_constants::SettingsMenuText::VOLUME_UP),
                            TextFont {
                                font: font.clone(),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(palette::TITLE),
                        ));
                    });
                });

            section.spawn((
                Text::new(crate::systems::text_constants::SettingsMenuText::VOLUME_HINT),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(palette::MUTED_LABEL),
            ));
        });
}
