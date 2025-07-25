use bevy::prelude::*;
use crate::{
    resources::*,
    states::*,
    components::*,
};

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

/// 设置游戏内 HUD
pub fn setup_game_hud(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
) {
    // 创建 HUD 根节点
    commands.spawn((
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
    )).with_children(|parent| {
        // 分数显示
        if let Some(assets) = &game_assets {
            parent.spawn((
                Text::new("Score: 0"),
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
                Text::new("Score: 0"),
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
            Text::new("Distance: 0m"),
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
            Text::new("WASD/Arrow Keys: Move | ESC: Pause"),
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
    // 更新分数显示
    if let Ok(mut score_text) = score_query.single_mut() {
        let score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
        **score_text = format!("Score: {}", score);
    }
    
    // 更新距离显示
    if let Ok(mut distance_text) = distance_query.single_mut() {
        **distance_text = format!("Distance: {}m", game_stats.distance_traveled as u32);
    }
}

/// 清理游戏 HUD
pub fn cleanup_game_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 暂停菜单组件
#[derive(Component)]
pub struct PauseMenu;

/// 设置增强的暂停菜单
pub fn setup_pause_menu(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
) {
    let font_handle = game_assets.as_ref().map(|a| a.font.clone()).unwrap_or_default();
    
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
        PauseMenuRoot,
    )).with_children(|parent| {
        parent.spawn((
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
            BorderColor(Color::WHITE),
        )).with_children(|parent| {
            // 游戏暂停标题
            parent.spawn((
                Text::new("Game Paused"),
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
            parent.spawn((
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
                BorderColor(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                ResumeButton,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Resume Game"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
            
            // 保存游戏按钮
            parent.spawn((
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
                BorderColor(Color::srgba(0.3, 0.3, 0.6, 1.0)),
                SaveGameButton,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Save Game"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
            
            // 主菜单按钮
            parent.spawn((
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
                BorderColor(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                MainMenuButton,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Main Menu"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
            
            // 键盘快捷键按钮
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
                // ESC键按钮
                parent.spawn((
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
                    BorderColor(Color::srgba(0.4, 0.8, 0.4, 1.0)),
                    EscKeyButton,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("ESC: Resume"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
                
                // Q键按钮
                parent.spawn((
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
                    BorderColor(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                    QKeyButton,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("Q: Main Menu"),
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
pub fn cleanup_pause_menu(
    mut commands: Commands,
    pause_query: Query<Entity, With<PauseMenuRoot>>,
) {
    for entity in pause_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 设置保存对话框
pub fn setup_save_dialog(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    mut save_name_input: ResMut<SaveNameInput>,
) {
    // 重置输入状态 - 清空输入框
    save_name_input.current_name.clear();
    save_name_input.is_editing = true;
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
        SaveDialog,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(250.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            BorderColor(Color::WHITE),
        )).with_children(|parent| {
            let font_handle = game_assets.as_ref().map(|a| a.font.clone()).unwrap_or_default();
            
            // 标题
            parent.spawn((
                Text::new("Save Game"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            
            // 输入提示
            parent.spawn((
                Text::new("Enter save name:"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
            
            // 输入框 (显示当前输入的名称)
            parent.spawn((
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                SaveNameInputBox,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(if save_name_input.current_name.is_empty() {
                        "Enter name...".to_string()
                    } else {
                        save_name_input.current_name.clone()
                    }),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(if save_name_input.current_name.is_empty() {
                        Color::srgba(0.7, 0.7, 0.7, 1.0) // 占位符颜色
                    } else {
                        Color::WHITE
                    }),
                ));
            });
            
            // 按钮容器
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|parent| {
                // 确认按钮
                parent.spawn((
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
                    BorderColor(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                    ConfirmSaveButton,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("Save"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
                
                // 取消按钮
                parent.spawn((
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
                    BorderColor(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                    CancelSaveButton,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("Cancel"),
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
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理存档名称输入
pub fn handle_save_name_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut save_name_input: ResMut<SaveNameInput>,
    mut text_query: Query<&mut Text, With<SaveNameInputBox>>,
) {
    if !save_name_input.is_editing {
        return;
    }
    
    let mut name_changed = false;
    
    // 处理字母和数字输入
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::KeyA => { save_name_input.current_name.push('A'); name_changed = true; }
            KeyCode::KeyB => { save_name_input.current_name.push('B'); name_changed = true; }
            KeyCode::KeyC => { save_name_input.current_name.push('C'); name_changed = true; }
            KeyCode::KeyD => { save_name_input.current_name.push('D'); name_changed = true; }
            KeyCode::KeyE => { save_name_input.current_name.push('E'); name_changed = true; }
            KeyCode::KeyF => { save_name_input.current_name.push('F'); name_changed = true; }
            KeyCode::KeyG => { save_name_input.current_name.push('G'); name_changed = true; }
            KeyCode::KeyH => { save_name_input.current_name.push('H'); name_changed = true; }
            KeyCode::KeyI => { save_name_input.current_name.push('I'); name_changed = true; }
            KeyCode::KeyJ => { save_name_input.current_name.push('J'); name_changed = true; }
            KeyCode::KeyK => { save_name_input.current_name.push('K'); name_changed = true; }
            KeyCode::KeyL => { save_name_input.current_name.push('L'); name_changed = true; }
            KeyCode::KeyM => { save_name_input.current_name.push('M'); name_changed = true; }
            KeyCode::KeyN => { save_name_input.current_name.push('N'); name_changed = true; }
            KeyCode::KeyO => { save_name_input.current_name.push('O'); name_changed = true; }
            KeyCode::KeyP => { save_name_input.current_name.push('P'); name_changed = true; }
            KeyCode::KeyQ => { save_name_input.current_name.push('Q'); name_changed = true; }
            KeyCode::KeyR => { save_name_input.current_name.push('R'); name_changed = true; }
            KeyCode::KeyS => { save_name_input.current_name.push('S'); name_changed = true; }
            KeyCode::KeyT => { save_name_input.current_name.push('T'); name_changed = true; }
            KeyCode::KeyU => { save_name_input.current_name.push('U'); name_changed = true; }
            KeyCode::KeyV => { save_name_input.current_name.push('V'); name_changed = true; }
            KeyCode::KeyW => { save_name_input.current_name.push('W'); name_changed = true; }
            KeyCode::KeyX => { save_name_input.current_name.push('X'); name_changed = true; }
            KeyCode::KeyY => { save_name_input.current_name.push('Y'); name_changed = true; }
            KeyCode::KeyZ => { save_name_input.current_name.push('Z'); name_changed = true; }
            KeyCode::Digit0 => { save_name_input.current_name.push('0'); name_changed = true; }
            KeyCode::Digit1 => { save_name_input.current_name.push('1'); name_changed = true; }
            KeyCode::Digit2 => { save_name_input.current_name.push('2'); name_changed = true; }
            KeyCode::Digit3 => { save_name_input.current_name.push('3'); name_changed = true; }
            KeyCode::Digit4 => { save_name_input.current_name.push('4'); name_changed = true; }
            KeyCode::Digit5 => { save_name_input.current_name.push('5'); name_changed = true; }
            KeyCode::Digit6 => { save_name_input.current_name.push('6'); name_changed = true; }
            KeyCode::Digit7 => { save_name_input.current_name.push('7'); name_changed = true; }
            KeyCode::Digit8 => { save_name_input.current_name.push('8'); name_changed = true; }
            KeyCode::Digit9 => { save_name_input.current_name.push('9'); name_changed = true; }
            KeyCode::Space => { save_name_input.current_name.push('_'); name_changed = true; }
            KeyCode::Minus => { save_name_input.current_name.push('-'); name_changed = true; }
            KeyCode::Backspace => {
                if !save_name_input.current_name.is_empty() {
                    save_name_input.current_name.pop();
                    name_changed = true;
                }
            }
            _ => {}
        }
    }
    
    // 限制名称长度
    if save_name_input.current_name.len() > 20 {
        save_name_input.current_name.truncate(20);
        name_changed = true;
    }
    
    // 更新显示的文本
    if name_changed {
        for mut text in text_query.iter_mut() {
            **text = if save_name_input.current_name.is_empty() {
                "Enter name...".to_string()
            } else {
                save_name_input.current_name.clone()
            };
        }
    }
}

/// 处理保存对话框交互
pub fn handle_save_dialog_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&ConfirmSaveButton>, Option<&CancelSaveButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut next_state: ResMut<NextState<GameState>>,
    pause_manager: Res<PauseManager>,
    mut save_file_manager: ResMut<SaveFileManager>,
    mut save_name_input: ResMut<SaveNameInput>,
) {
    let mut should_save = false;
    let mut should_cancel = false;
    
    for (interaction, mut color, confirm_btn, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if confirm_btn.is_some() {
                    should_save = true;
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
    
    if should_save {
        // 执行保存操作
        if let Some(state) = &pause_manager.preserved_state {
            let save_name = if save_name_input.current_name.is_empty() {
                "DefaultSave".to_string()
            } else {
                save_name_input.current_name.clone()
            };
            
            match crate::systems::pause_save::save_game_to_file(
                save_name.clone(),
                state.clone(),
                save_file_manager,
            ) {
                Ok(_) => {
                    println!("✅ 保存成功: {}", save_name);
                    save_name_input.is_editing = false;
                    next_state.set(GameState::Paused);
                }
                Err(e) => {
                    println!("❌ 保存失败: {}", e);
                    next_state.set(GameState::Paused);
                }
            }
        }
    } else if should_cancel {
        // 取消保存，返回暂停菜单
        save_name_input.is_editing = false;
        next_state.set(GameState::Paused);
        println!("❌ 取消保存");
    }
}

/// 设置加载表格界面
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
                width: Val::Px(600.0),
                height: Val::Px(500.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            BorderColor(Color::WHITE),
        )).with_children(|parent| {
            let font_handle = game_assets.as_ref().map(|a| a.font.clone()).unwrap_or_default();
            
            // 标题
            parent.spawn((
                Text::new("Load Game"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
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
                BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            )).with_children(|parent| {
                let headers = ["Name", "Score", "Distance", "Time", "Date"];
                let widths = [25.0, 15.0, 20.0, 15.0, 25.0];
                
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
                        BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new(*header),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 16.0,
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
                    height: Val::Px(300.0),
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
                            Text::new("No save files found"),
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
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(40.0),
                                flex_direction: FlexDirection::Row,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if index % 2 == 0 {
                                Color::srgba(0.25, 0.25, 0.25, 0.8)
                            } else {
                                Color::srgba(0.2, 0.2, 0.2, 0.8)
                            }),
                            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                            SaveFileRow { save_index: index },
                        )).with_children(|parent| {
                            let widths = [25.0, 15.0, 20.0, 15.0, 25.0];
                            let values = [
                                save_file.name.clone(),
                                save_file.score.to_string(),
                                format!("{:.1}m", save_file.distance),
                                format!("{:.1}s", save_file.play_time),
                                save_file.save_timestamp.format("%m/%d %H:%M").to_string(),
                            ];
                            
                            for (value, width) in values.iter().zip(widths.iter()) {
                                parent.spawn((
                                    Node {
                                        width: Val::Percent(*width),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::right(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                )).with_children(|parent| {
                                    parent.spawn((
                                        Text::new(value.clone()),
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
                // 返回按钮
                parent.spawn((
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
                    BorderColor(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                    CancelSaveButton, // 重用取消按钮组件
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("Back"),
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

/// 清理加载表格
pub fn cleanup_load_table(
    mut commands: Commands,
    table_query: Query<Entity, With<LoadTableRoot>>,
) {
    for entity in table_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理加载表格交互
pub fn handle_load_table_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SaveFileRow>, Option<&CancelSaveButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut next_state: ResMut<NextState<GameState>>,
    save_file_manager: Res<SaveFileManager>,
    mut loaded_game_state: ResMut<LoadedGameState>,
) {
    let mut selected_save_index: Option<usize> = None;
    let mut should_cancel = false;
    
    for (interaction, mut color, save_row, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(row) = save_row {
                    selected_save_index = Some(row.save_index);
                } else if cancel_btn.is_some() {
                    should_cancel = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if save_row.is_some() {
                    *color = BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.9));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                }
            }
            Interaction::None => {
                if let Some(row) = save_row {
                    *color = BackgroundColor(if row.save_index % 2 == 0 {
                        Color::srgba(0.25, 0.25, 0.25, 0.8)
                    } else {
                        Color::srgba(0.2, 0.2, 0.2, 0.8)
                    });
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                }
            }
        }
    }
    
    if let Some(index) = selected_save_index {
        if index < save_file_manager.save_files.len() {
            let save_file = &save_file_manager.save_files[index];
            match crate::systems::pause_save::load_game_from_file(&save_file.file_path) {
                Ok(game_state) => {
                    println!("📂 加载存档: {}", save_file.name);
                    loaded_game_state.state = Some(game_state);
                    loaded_game_state.should_restore = true;
                    next_state.set(GameState::Playing);
                }
                Err(e) => {
                    println!("❌ 加载失败: {}", e);
                }
            }
        }
    } else if should_cancel {
        // 返回主菜单
        next_state.set(GameState::Menu);
        println!("🏠 返回主菜单");
    }
}

/// 处理暂停菜单按钮交互
pub fn handle_pause_menu_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&ResumeButton>, Option<&SaveGameButton>, Option<&MainMenuButton>, Option<&EscKeyButton>, Option<&QKeyButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    _player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    _camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    _game_stats: Res<GameStats>,
    _character_selection: Res<CharacterSelection>,
) {
    for (interaction, mut color, resume_btn, save_btn, menu_btn, esc_btn, q_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if resume_btn.is_some() || esc_btn.is_some() {
                    // 继续游戏
                    next_state.set(GameState::Playing);
                    println!("▶️ 继续游戏");
                } else if save_btn.is_some() {
                    // 进入保存对话框
                    next_state.set(GameState::SaveDialog);
                    println!("💾 打开保存对话框");
                } else if menu_btn.is_some() || q_btn.is_some() {
                    // 返回主菜单
                    pause_manager.resume_game(); // 清理暂停状态
                    next_state.set(GameState::Menu);
                    println!("🏠 返回主菜单");
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if resume_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if save_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.25, 0.5, 0.9));
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