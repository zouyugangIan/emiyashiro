use crate::{components::*, resources::*, states::*};
use bevy::prelude::*;

/// 游戏内 HUD 组件
#[derive(Component)]
pub struct GameHUD;

/// 分数显示组件
#[derive(Component)]
pub struct ScoreDisplay;

/// 距离显示组件
#[derive(Component)]
pub struct DistanceDisplay;

// Enhanced Pause System UI Components
#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SaveGameButton;

#[derive(Component)]
pub struct LoadGameButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct MainMenuButton;

// 键盘提示按钮组件
#[derive(Component)]
pub struct EscKeyButton;

#[derive(Component)]
pub struct QKeyButton;

// 存档名称输入资源
#[derive(Resource, Default)]
pub struct SaveNameInput {
    pub current_name: String,
    pub is_editing: bool,
}

// 加载的游戏状态资源
#[derive(Resource, Default)]
pub struct LoadedGameState {
    pub state: Option<CompleteGameState>,
    pub should_restore: bool,
    pub previous_state: Option<GameState>,
}

// Save Dialog Components
#[derive(Component)]
pub struct SaveDialog;

#[derive(Component)]
pub struct ConfirmSaveButton;

#[derive(Component)]
pub struct CancelSaveButton;

#[derive(Component)]
pub struct SaveNameInputBox;

#[derive(Component)]
pub struct TextCursor {
    pub blink_timer: f32,
    pub visible: bool,
}

// Load Table Components
#[derive(Component)]
pub struct LoadTableRoot;

#[derive(Component)]
pub struct SaveFileRow {
    pub save_index: usize,
}

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct DeleteButton;

#[derive(Component)]
pub struct RenameButton;

#[derive(Component)]
pub struct RenameDialog;

#[derive(Component)]
pub struct RenameInputBox;

#[derive(Component)]
pub struct ConfirmRenameButton;

#[derive(Component)]
pub struct CancelRenameButton;

// 重命名输入资源
#[derive(Resource, Default)]
pub struct RenameInput {
    pub current_name: String,
    pub original_name: String,
    pub save_index: usize,
    pub is_editing: bool,
}

/// 设置游戏内 HUD
pub fn setup_game_hud(mut commands: Commands, game_assets: Option<Res<GameAssets>>) {
    // 创建 HUD 根节点
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            GameHUD,
        ))
        .with_children(|parent| {
            // 分数显示
            if let Some(assets) = &game_assets {
                parent.spawn((
                    Text::new(format!(
                        "{}0",
                        crate::systems::text_constants::GameHUDText::SCORE_LABEL
                    )),
                    TextFont {
                        font: assets.font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ScoreDisplay,
                ));
            } else {
                parent.spawn((
                    Text::new(format!(
                        "{}0",
                        crate::systems::text_constants::GameHUDText::SCORE_LABEL
                    )),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ScoreDisplay,
                ));
            }

            // 距离显示
            parent.spawn((
                Text::new(format!(
                    "{}0{}",
                    crate::systems::text_constants::GameHUDText::DISTANCE_LABEL,
                    crate::systems::text_constants::GameHUDText::METERS_UNIT
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                DistanceDisplay,
            ));

            // 操作提示
            parent.spawn((
                Text::new(crate::systems::text_constants::PauseMenuText::CONTROLS_HINT),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

/// 更新游戏 HUD
pub fn update_game_hud(
    mut score_query: Query<&mut Text, (With<ScoreDisplay>, Without<DistanceDisplay>)>,
    mut distance_query: Query<&mut Text, (With<DistanceDisplay>, Without<ScoreDisplay>)>,
    game_stats: Res<GameStats>,
) {
    use crate::systems::text_constants::GameHUDText;

    // 更新分数显示
    if let Ok(mut score_text) = score_query.single_mut() {
        let score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
        **score_text = format!("{}{}", GameHUDText::SCORE_LABEL, score);
    }

    // 更新距离显示
    if let Ok(mut distance_text) = distance_query.single_mut() {
        **distance_text = format!(
            "{}{}{}",
            GameHUDText::DISTANCE_LABEL,
            game_stats.distance_traveled as u32,
            GameHUDText::METERS_UNIT
        );
    }
}

/// 清理游戏 HUD
pub fn cleanup_game_hud(mut commands: Commands, hud_query: Query<Entity, With<GameHUD>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 暂停菜单组件
#[derive(Component)]
pub struct PauseMenu;

/// 设置增强的暂停菜单
pub fn setup_pause_menu(mut commands: Commands, game_assets: Option<Res<GameAssets>>) {
    use crate::systems::text_constants::PauseMenuText;

    let font_handle = game_assets
        .as_ref()
        .map(|a| a.font.clone())
        .unwrap_or_default();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(350.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    // 游戏暂停标题
                    parent.spawn((
                        Text::new(PauseMenuText::TITLE),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // 继续游戏按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                            ResumeButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::RESUME_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 保存游戏按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.3, 0.6, 1.0)),
                            SaveGameButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::SAVE_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 加载游戏按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                            LoadGameButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::LOAD_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 主菜单按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                            MainMenuButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::MAIN_MENU),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 键盘快捷键按钮
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // ESC键按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.4, 0.8, 0.4, 1.0)),
                                    EscKeyButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(PauseMenuText::ESC_RESUME),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // Q键按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                                    QKeyButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(PauseMenuText::Q_MAIN_MENU),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });
}

/// 清理暂停菜单
pub fn cleanup_pause_menu(mut commands: Commands, pause_query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in pause_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 设置保存对话框
pub fn setup_save_dialog(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    mut save_name_input: ResMut<SaveNameInput>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    use crate::systems::text_constants::SaveLoadText;

    // 重置输入状态 - 清空输入框
    save_name_input.current_name.clear();
    save_name_input.is_editing = true;

    // 激活新的文本输入系统
    text_input_state.activate();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            SaveDialog,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        height: Val::Px(280.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    let font_handle = game_assets
                        .as_ref()
                        .map(|a| a.font.clone())
                        .unwrap_or_default();

                    // 标题
                    parent.spawn((
                        Text::new(SaveLoadText::SAVE_DIALOG_TITLE),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 输入提示
                    parent.spawn((
                        Text::new(SaveLoadText::ENTER_SAVE_NAME),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));

                    // 输入框 (显示当前输入的名称)
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(350.0),
                                height: Val::Px(45.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::srgba(0.5, 0.8, 1.0, 1.0)), // Blue border to indicate active input
                        ))
                        .with_children(|parent| {
                            // 输入文本
                            parent.spawn((
                                Text::new(SaveLoadText::NAME_PLACEHOLDER),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)), // Placeholder color
                                SaveNameInputBox, // 将标记添加到文本组件上
                            ));

                            // 光标
                            parent.spawn((
                                Text::new("|"),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                                TextCursor {
                                    blink_timer: 0.0,
                                    visible: true,
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(15.0), // 初始位置，会根据文本长度动态调整
                                    ..default()
                                },
                            ));
                        });

                    // 输入提示信息
                    parent.spawn((
                        Text::new(SaveLoadText::INPUT_HINT),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.6)),
                    ));

                    // 按钮容器
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // 确认按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                                    ConfirmSaveButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(SaveLoadText::SAVE_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // 取消按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                                    CancelSaveButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(SaveLoadText::CANCEL_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });
}

/// 清理保存对话框
pub fn cleanup_save_dialog(
    mut commands: Commands,
    dialog_query: Query<Entity, With<SaveDialog>>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // 停用文本输入系统
    text_input_state.deactivate();
}

/// Text cursor blinking system
pub fn update_text_cursor(
    time: Res<Time>,
    mut cursor_query: Query<(&mut TextCursor, &mut Visibility, &mut Node)>,
    save_name_input: Res<SaveNameInput>,
) {
    for (mut cursor, mut visibility, mut node) in cursor_query.iter_mut() {
        cursor.blink_timer += time.delta_secs();

        if cursor.blink_timer >= 0.5 {
            cursor.blink_timer = 0.0;
            cursor.visible = !cursor.visible;
            *visibility = if cursor.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        // 更新光标位置基于当前文本长度
        // 每个字符大约8像素宽度（16号字体的估算）
        let text_width = save_name_input.current_name.len() as f32 * 8.0;
        node.left = Val::Px(15.0 + text_width);
    }
}

/// Enhanced text input handler for save names
pub fn handle_save_name_input(
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    mut save_name_input: ResMut<SaveNameInput>,
    mut text_query: Query<&mut Text, With<SaveNameInputBox>>,
) {
    // 同步新的文本输入系统状态到旧的保存名称输入
    if text_input_state.is_active {
        save_name_input.current_name = text_input_state.current_text.clone();
        save_name_input.is_editing = true;

        // 更新显示文本
        use crate::systems::text_constants::SaveLoadText;
        for mut text in text_query.iter_mut() {
            text.0 = if text_input_state.current_text.is_empty() {
                SaveLoadText::NAME_PLACEHOLDER.to_string()
            } else {
                format!("{}|", text_input_state.current_text)
            };
        }
    } else {
        save_name_input.is_editing = false;
    }
}

/// Map keyboard input to characters
fn map_keycode_to_char(keycode: &KeyCode) -> Option<char> {
    match keycode {
        // Letters - convert to uppercase
        KeyCode::KeyA => Some('A'),
        KeyCode::KeyB => Some('B'),
        KeyCode::KeyC => Some('C'),
        KeyCode::KeyD => Some('D'),
        KeyCode::KeyE => Some('E'),
        KeyCode::KeyF => Some('F'),
        KeyCode::KeyG => Some('G'),
        KeyCode::KeyH => Some('H'),
        KeyCode::KeyI => Some('I'),
        KeyCode::KeyJ => Some('J'),
        KeyCode::KeyK => Some('K'),
        KeyCode::KeyL => Some('L'),
        KeyCode::KeyM => Some('M'),
        KeyCode::KeyN => Some('N'),
        KeyCode::KeyO => Some('O'),
        KeyCode::KeyP => Some('P'),
        KeyCode::KeyQ => Some('Q'),
        KeyCode::KeyR => Some('R'),
        KeyCode::KeyS => Some('S'),
        KeyCode::KeyT => Some('T'),
        KeyCode::KeyU => Some('U'),
        KeyCode::KeyV => Some('V'),
        KeyCode::KeyW => Some('W'),
        KeyCode::KeyX => Some('X'),
        KeyCode::KeyY => Some('Y'),
        KeyCode::KeyZ => Some('Z'),
        // Numbers
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        // Special characters
        KeyCode::Space => Some('_'),
        KeyCode::Minus => Some('-'),
        _ => None,
    }
}

/// Update the input display text
fn update_input_display(
    save_name_input: &SaveNameInput,
    mut text_query: Query<&mut Text, With<SaveNameInputBox>>,
) {
    use crate::systems::text_constants::SaveLoadText;
    for mut text in text_query.iter_mut() {
        text.0 = if save_name_input.current_name.is_empty() {
            SaveLoadText::NAME_PLACEHOLDER.to_string()
        } else {
            format!("{}|", save_name_input.current_name) // Add cursor indicator
        };
    }
}

/// 处理保存对话框交互
pub fn handle_save_dialog_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ConfirmSaveButton>,
            Option<&CancelSaveButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    pause_manager: Res<PauseManager>,
    save_file_manager: ResMut<SaveFileManager>,
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    use crate::systems::text_constants::SaveLoadText;

    let mut should_save = false;
    let mut should_cancel = false;

    // 处理键盘快捷键
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        should_save = true;
        println!("💾 Enter key pressed - saving game");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        should_cancel = true;
        println!("❌ Escape key pressed - canceling save");
    }

    // 处理按钮交互
    for (interaction, mut color, confirm_btn, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if confirm_btn.is_some() {
                    println!("💾 Save button pressed!");
                    should_save = true;
                } else if cancel_btn.is_some() {
                    println!("❌ Cancel button pressed!");
                    should_cancel = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                }
            }
            Interaction::None => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                }
            }
        }
    }

    if should_save {
        println!("💾 Attempting to save game...");
        // 执行保存操作
        if let Some(state) = &pause_manager.preserved_state {
            let save_name = if text_input_state.current_text.is_empty() {
                SaveLoadText::DEFAULT_SAVE_NAME.to_string()
            } else {
                text_input_state.current_text.clone()
            };

            println!("💾 Saving with name: '{}', state exists: true", save_name);

            match crate::systems::pause_save::save_game_to_file(
                save_name.clone(),
                state.clone(),
                save_file_manager,
            ) {
                Ok(_) => {
                    println!("✅ {}: {}", SaveLoadText::SAVE_SUCCESS, save_name);
                    // 保存成功后返回暂停菜单，而不是跳转到加载界面
                    next_state.set(GameState::Paused);
                }
                Err(e) => {
                    println!("❌ {}: {:?}", SaveLoadText::SAVE_ERROR, e);
                    next_state.set(GameState::Paused);
                }
            }
        } else {
            println!("❌ No game state to save! PauseManager preserved_state is None");
            // 如果没有保存的状态，尝试直接从当前游戏状态创建一个
            println!("🔄 Attempting to create current game state for saving...");
            next_state.set(GameState::Paused);
        }
    } else if should_cancel {
        // 取消保存，返回暂停菜单
        next_state.set(GameState::Paused);
        println!("❌ Save canceled");
    }
}

/// 设置增强的加载表格界面
pub fn setup_load_table(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    save_file_manager: Res<SaveFileManager>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        LoadTableRoot,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(700.0),
                height: Val::Px(550.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            BorderColor::all(Color::WHITE),
        )).with_children(|parent| {
            let font_handle = game_assets.as_ref().map(|a| a.font.clone()).unwrap_or_default();

            // 标题
            parent.spawn((
                Text::new(crate::systems::text_constants::SaveLoadText::LOAD_DIALOG_TITLE),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 操作提示
            parent.spawn((
                Text::new(crate::systems::text_constants::SaveLoadText::CLICK_TO_LOAD),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 表格头部
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
                BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            )).with_children(|parent| {
                use crate::systems::text_constants::SaveLoadText;
                let headers = [
                    SaveLoadText::COL_NAME,
                    SaveLoadText::COL_PLAYERS,
                    SaveLoadText::COL_SCORE,
                    SaveLoadText::COL_DISTANCE,
                    SaveLoadText::COL_TIME,
                    SaveLoadText::COL_DATE,
                    SaveLoadText::COL_ACTIONS
                ];
                let widths = [18.0, 8.0, 12.0, 12.0, 12.0, 18.0, 20.0];

                for (header, width) in headers.iter().zip(widths.iter()) {
                    parent.spawn((
                        Node {
                            width: Val::Percent(*width),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::right(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new(*header),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // 滚动区域
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(320.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            )).with_children(|parent| {
                // 显示存档文件
                if save_file_manager.save_files.is_empty() {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new(crate::systems::text_constants::SaveLoadText::NO_SAVES_FOUND),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        ));
                    });
                } else {
                    for (index, save_file) in save_file_manager.save_files.iter().enumerate() {
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(45.0),
                                flex_direction: FlexDirection::Row,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if index % 2 == 0 {
                                Color::srgba(0.25, 0.25, 0.25, 0.8)
                            } else {
                                Color::srgba(0.2, 0.2, 0.2, 0.8)
                            }),
                            BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                        )).with_children(|parent| {
                            let widths = [18.0, 8.0, 12.0, 12.0, 12.0, 18.0, 20.0];
                            let values = [
                                save_file.name.clone(),
                                "1P".to_string(), // 默认单人游戏，未来可从存档数据读取
                                save_file.score.to_string(),
                                format!("{:.1}m", save_file.distance),
                                format!("{:.1}s", save_file.play_time),
                                save_file.save_timestamp.format("%m/%d %H:%M").to_string(),
                            ];

                            // 显示存档信息
                            for (i, (value, width)) in values.iter().zip(widths.iter()).enumerate() {
                                if i < 6 { // 前6列显示数据
                                    parent.spawn((
                                        Button,
                                        Node {
                                            width: Val::Percent(*width),
                                            height: Val::Percent(100.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::right(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                        SaveFileRow { save_index: index },
                                    )).with_children(|parent| {
                                        parent.spawn((
                                            Text::new(value.clone()),
                                            TextFont {
                                                font: font_handle.clone(),
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(if i == 1 { // 玩家数量列使用不同颜色
                                                Color::srgba(0.7, 0.9, 1.0, 1.0)
                                            } else {
                                                Color::WHITE
                                            }),
                                        ));
                                    });
                                }
                            }

                            // 操作按钮列
                            parent.spawn((
                                Node {
                                    width: Val::Percent(20.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceEvenly,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                            )).with_children(|parent| {
                                // 重命名按钮
                                parent.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(45.0),
                                        height: Val::Px(25.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.3, 0.4, 0.8)),
                                    BorderColor::all(Color::srgba(0.4, 0.6, 0.8, 1.0)),
                                    RenameButton,
                                    SaveFileRow { save_index: index },
                                )).with_children(|parent| {
                                    parent.spawn((
                                        Text::new(crate::systems::text_constants::SaveLoadText::RENAME_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                                // 删除按钮
                                parent.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(45.0),
                                        height: Val::Px(25.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                                    DeleteButton,
                                    SaveFileRow { save_index: index },
                                )).with_children(|parent| {
                                    parent.spawn((
                                        Text::new(crate::systems::text_constants::SaveLoadText::DELETE_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            });
                        });
                    }
                }
            });

            // 底部按钮
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|parent| {
                // 刷新按钮
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.25, 0.35, 0.8)),
                    BorderColor::all(Color::srgba(0.3, 0.5, 0.7, 1.0)),
                    LoadButton, // 重用加载按钮组件作为刷新
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::REFRESH_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                // 返回按钮
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                    CancelSaveButton, // 重用取消按钮组件
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::BACK_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
    });
}

/// 清理加载表格
pub fn cleanup_load_table(mut commands: Commands, table_query: Query<Entity, With<LoadTableRoot>>) {
    for entity in table_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 设置重命名对话框
pub fn setup_rename_dialog(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    mut rename_input: ResMut<RenameInput>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    // 重置输入状态，使用原始名称作为默认值
    rename_input.current_name = rename_input.original_name.clone();
    rename_input.is_editing = true;

    // 激活文本输入系统并设置初始值
    text_input_state.activate();
    text_input_state.current_text = rename_input.original_name.clone();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            RenameDialog,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        height: Val::Px(280.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    let font_handle = game_assets
                        .as_ref()
                        .map(|a| a.font.clone())
                        .unwrap_or_default();

                    // 标题
                    parent.spawn((
                        Text::new(
                            crate::systems::text_constants::SaveLoadText::RENAME_DIALOG_TITLE,
                        ),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 原始名称显示
                    parent.spawn((
                        Text::new(format!("Current name: {}", rename_input.original_name)),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    ));

                    // 输入提示
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::ENTER_NEW_NAME),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));

                    // 输入框
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(350.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                            RenameInputBox,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(rename_input.current_name.clone()),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 按钮容器
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // 确认按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                                    ConfirmRenameButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::CONFIRM_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                                });

                            // 取消按钮
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                                    CancelRenameButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::CANCEL_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                                });
                        });
                });
        });
}

/// 清理重命名对话框
pub fn cleanup_rename_dialog(
    mut commands: Commands,
    dialog_query: Query<Entity, With<RenameDialog>>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // 停用文本输入系统
    text_input_state.deactivate();
}

/// 处理增强的加载表格交互
pub fn handle_load_table_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&SaveFileRow>,
            Option<&CancelSaveButton>,
            Option<&LoadButton>,
            Option<&RenameButton>,
            Option<&DeleteButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_file_manager: ResMut<SaveFileManager>,
    mut loaded_game_state: ResMut<LoadedGameState>,
    mut rename_input: ResMut<RenameInput>,
) {
    let mut selected_save_index: Option<usize> = None;
    let mut should_cancel = false;
    let mut should_refresh = false;
    let mut rename_index: Option<usize> = None;
    let mut delete_index: Option<usize> = None;

    for (interaction, mut color, save_row, cancel_btn, load_btn, rename_btn, delete_btn) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(&SaveFileRow { save_index }) = save_row {
                    if rename_btn.is_some() {
                        rename_index = Some(save_index);
                    } else if delete_btn.is_some() {
                        delete_index = Some(save_index);
                    } else {
                        selected_save_index = Some(save_index);
                    }
                } else if cancel_btn.is_some() {
                    should_cancel = true;
                } else if load_btn.is_some() {
                    should_refresh = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if save_row.is_some() {
                    if rename_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.3, 0.45, 0.6, 0.9));
                    } else if delete_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.6, 0.3, 0.3, 0.9));
                    } else {
                        *color = BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.9));
                    }
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.4, 0.55, 0.9));
                }
            }
            Interaction::None => {
                if let Some(_row) = save_row {
                    if rename_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.2, 0.3, 0.4, 0.8));
                    } else if delete_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8));
                    } else {
                        *color = BackgroundColor(Color::NONE);
                    }
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.25, 0.35, 0.8));
                }
            }
        }
    }

    // 处理加载存档
    if let Some(index) = selected_save_index {
        if index < save_file_manager.save_files.len() {
            let save_file = &save_file_manager.save_files[index];
            match crate::systems::pause_save::load_game_from_file(&save_file.file_path) {
                Ok(game_state) => {
                    println!(
                        "📂 {}: {}",
                        crate::systems::text_constants::SaveLoadText::LOAD_SUCCESS,
                        save_file.name
                    );
                    loaded_game_state.state = Some(game_state);
                    loaded_game_state.should_restore = true;
                    next_state.set(GameState::Playing);
                }
                Err(e) => {
                    println!(
                        "❌ {}: {}",
                        crate::systems::text_constants::SaveLoadText::LOAD_ERROR,
                        e
                    );
                }
            }
        }
    }
    // 处理重命名
    else if let Some(index) = rename_index {
        if index < save_file_manager.save_files.len() {
            let save_file = &save_file_manager.save_files[index];
            rename_input.original_name = save_file.name.clone();
            rename_input.save_index = index;
            next_state.set(GameState::RenameDialog);
            println!("✏️ Renaming save: {}", save_file.name);
        }
    }
    // 处理删除
    else if let Some(index) = delete_index {
        if index < save_file_manager.save_files.len() {
            let save_name = save_file_manager.save_files[index].name.clone();
            match crate::systems::pause_save::delete_save_file(&save_name, &mut save_file_manager) {
                Ok(_) => {
                    println!(
                        "🗑️ {}: {}",
                        crate::systems::text_constants::SaveLoadText::DELETE_SUCCESS,
                        save_name
                    );
                    // 刷新存档列表
                    should_refresh = true;
                }
                Err(e) => {
                    println!(
                        "❌ {}: {}",
                        crate::systems::text_constants::SaveLoadText::DELETE_ERROR,
                        e
                    );
                }
            }
        }
    }
    // 处理刷新
    else if should_refresh {
        // 触发存档文件扫描并重新加载UI
        crate::systems::pause_save::scan_save_files(save_file_manager);
        next_state.set(GameState::LoadTable);
        println!("🔄 Refreshing save list and reloading UI");
    }
    // 处理返回
    else if should_cancel {
        // 根据来源状态返回到正确的地方
        let target_state = loaded_game_state
            .previous_state
            .clone()
            .unwrap_or(GameState::Menu);
        next_state.set(target_state.clone());
        loaded_game_state.previous_state = None; // 清理状态

        match target_state {
            GameState::Menu => println!("🏠 Back to main menu"),
            GameState::Paused => println!("⏸️ Back to pause menu"),
            _ => println!("🔙 Back to previous state"),
        }
    }

    // Explicitly use should_refresh to silence the warning, as the compiler seems to miss its usage.
    let _ = should_refresh;
}

/// Enhanced text input handler for rename operations
pub fn handle_rename_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut rename_input: ResMut<RenameInput>,
    mut text_query: Query<&mut Text, With<RenameInputBox>>,
    text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    if !rename_input.is_editing {
        return;
    }

    // 同步新的文本输入系统状态到重命名输入
    if text_input_state.is_active {
        rename_input.current_name = text_input_state.current_text.clone();

        // 更新显示文本
        for mut text in text_query.iter_mut() {
            text.0 = if text_input_state.current_text.is_empty() {
                crate::systems::text_constants::SaveLoadText::NAME_PLACEHOLDER.to_string()
            } else {
                format!("{}|", text_input_state.current_text)
            };
        }
    }

    // 处理键盘快捷键
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        println!("✏️ Enter key pressed - confirming rename");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        println!("❌ Escape key pressed - canceling rename");
    }
}

/// Update the rename input display text
fn update_rename_display(
    rename_input: &RenameInput,
    mut text_query: Query<&mut Text, With<RenameInputBox>>,
) {
    for mut text in text_query.iter_mut() {
        text.0 = if rename_input.current_name.is_empty() {
            "Enter name...".to_string()
        } else {
            format!("{}|", rename_input.current_name) // Add cursor indicator
        };
    }
}

/// 处理重命名对话框交互
pub fn handle_rename_dialog_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ConfirmRenameButton>,
            Option<&CancelRenameButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_file_manager: ResMut<SaveFileManager>,
    mut rename_input: ResMut<RenameInput>,
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut should_confirm = false;
    let mut should_cancel = false;

    // 处理键盘快捷键
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        should_confirm = true;
        println!("✏️ Enter key pressed - confirming rename");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        should_cancel = true;
        println!("❌ Escape key pressed - canceling rename");
    }

    // 处理按钮交互
    for (interaction, mut color, confirm_btn, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if confirm_btn.is_some() {
                    should_confirm = true;
                } else if cancel_btn.is_some() {
                    should_cancel = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                }
            }
            Interaction::None => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                }
            }
        }
    }

    if should_confirm {
        // 执行重命名操作
        let new_name = if text_input_state.current_text.is_empty() {
            rename_input.original_name.clone()
        } else {
            text_input_state.current_text.clone()
        };

        if rename_input.save_index < save_file_manager.save_files.len() {
            match crate::systems::pause_save::rename_save_file(
                &rename_input.original_name,
                &new_name,
                save_file_manager.as_mut(),
            ) {
                Ok(_) => {
                    println!(
                        "✅ {}: {} -> {}",
                        crate::systems::text_constants::SaveLoadText::RENAME_SUCCESS,
                        rename_input.original_name,
                        new_name
                    );
                    rename_input.is_editing = false;
                    next_state.set(GameState::LoadTable);
                }
                Err(e) => {
                    println!(
                        "❌ {}: {}",
                        crate::systems::text_constants::SaveLoadText::RENAME_ERROR,
                        e
                    );
                    next_state.set(GameState::LoadTable);
                }
            }
        }
    } else if should_cancel {
        // 取消重命名，返回加载表格
        rename_input.is_editing = false;
        next_state.set(GameState::LoadTable);
        println!("❌ Rename cancelled");
    }
}

/// 处理暂停菜单按钮交互
pub fn handle_pause_menu_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ResumeButton>,
            Option<&SaveGameButton>,
            Option<&LoadGameButton>,
            Option<&MainMenuButton>,
            Option<&EscKeyButton>,
            Option<&QKeyButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    mut loaded_game_state: ResMut<LoadedGameState>,
    _player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    _camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    _game_stats: Res<GameStats>,
    _character_selection: Res<CharacterSelection>,
) {
    for (interaction, mut color, resume_btn, save_btn, load_btn, menu_btn, esc_btn, q_btn) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if resume_btn.is_some() || esc_btn.is_some() {
                    // 继续游戏
                    next_state.set(GameState::Playing);
                    println!("▶️ Resume game");
                } else if save_btn.is_some() {
                    // 进入保存对话框
                    next_state.set(GameState::SaveDialog);
                    println!("💾 Open save dialog");
                } else if load_btn.is_some() {
                    // 进入加载表格，记录来源状态
                    loaded_game_state.previous_state = Some(GameState::Paused);
                    next_state.set(GameState::LoadTable);
                    println!("📂 Open load table from pause menu");
                } else if menu_btn.is_some() || q_btn.is_some() {
                    // 返回主菜单
                    pause_manager.resume_game(); // 清理暂停状态
                    next_state.set(GameState::Menu);
                    println!("🏠 Return to main menu");
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if resume_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if save_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.25, 0.5, 0.9));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if menu_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                } else if esc_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.6, 0.3, 0.9));
                } else if q_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.6, 0.3, 0.3, 0.9));
                }
            }
            Interaction::None => {
                if resume_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if save_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if menu_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                } else if esc_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.8));
                } else if q_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8));
                }
            }
        }
    }
}
