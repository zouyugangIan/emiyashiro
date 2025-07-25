//! 主菜单系统
//! 
//! 包含主菜单界面的创建、交互处理和动画效果。

use bevy::prelude::*;
use crate::{
    components::*,
    states::*,
    resources::*,
    systems::ui::LoadButton,
};

/// 设置主菜单界面
/// 
/// 创建主菜单的UI元素，包括标题、按钮、背景图片等。
/// 支持角色选择和封面图片渐变动画。
/// 
/// # 参数
/// * `commands` - 用于生成实体的命令缓冲区
/// * `game_assets` - 游戏资源句柄（可选）
/// * `camera_query` - 摄像机查询
pub fn setup_menu(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    // 只有在没有摄像机时才创建
    if camera_query.is_empty() {
        commands.spawn(Camera2d);
    }
    
    // 如果资源已加载，创建封面背景渐变效果
    if let Some(ref assets) = game_assets {
        // 第一张封面图片 - 调整到游戏界面大小
        commands.spawn((
            Sprite {
                image: assets.cover_texture.clone(),
                custom_size: Some(Vec2::new(1024.0, 768.0)), // 匹配游戏窗口大小
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            MenuUI,
            CoverImage1,
            CoverFadeState::default(),
        ));
        
        // 第二张封面图片 - 调整到游戏界面大小，从透明开始
        commands.spawn((
            Sprite {
                image: assets.cover2_texture.clone(), // 使用正确的第二张封面
                custom_size: Some(Vec2::new(1024.0, 768.0)), // 匹配游戏窗口大小
                color: Color::srgba(1.0, 1.0, 1.0, 0.0), // 从透明开始
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)), // 稍微前置
            MenuUI,
            CoverImage2,
            CoverFadeState { 
                alpha: 0.0, // 从0.0开始
                fade_direction: -1.0, // 负方向表示第二张图片
            },
        ));
    } else {
        // 创建简单的背景色
        commands.spawn((
            Sprite {
                color: Color::srgb(0.1, 0.1, 0.2),
                custom_size: Some(Vec2::new(1024.0, 768.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            MenuUI,
        ));
    }
    
    // 创建UI根节点
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        MenuUI,
    )).with_children(|parent| {
        // 游戏标题 - 使用英文避免字体问题
        if let Some(assets) = &game_assets {
            parent.spawn((
                Text::new("Fate/stay night Heaven's Feel\nShirou Runner"),
                TextFont {
                    font: assets.font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
        } else {
            parent.spawn((
                Text::new("Fate/stay night Heaven's Feel\nShirou Runner"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
        }
        
        // 按钮容器
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        )).with_children(|parent| {
            // 开始按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                StartButton,
            )).with_children(|parent| {
                if let Some(assets) = &game_assets {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font: assets.font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                } else {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
            
            // 加载存档按钮
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
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8)),
                LoadButton,
            )).with_children(|parent| {
                if let Some(assets) = &game_assets {
                    parent.spawn((
                        Text::new("Load Game"),
                        TextFont {
                            font: assets.font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                } else {
                    parent.spawn((
                        Text::new("Load Game"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        });
        
        // 角色选择按钮
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        )).with_children(|parent| {
            // 角色1按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8)),
                CharacterSelectButton {
                    character_type: CharacterType::Shirou1,
                },
            )).with_children(|parent| {
                if let Some(assets) = &game_assets {
                    parent.spawn((
                        Text::new("Shirou 1P"),
                        TextFont {
                            font: assets.font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                } else {
                    parent.spawn((
                        Text::new("Shirou 1P"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
            
            // 角色2按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 0.8)),
                CharacterSelectButton {
                    character_type: CharacterType::Shirou2,
                },
            )).with_children(|parent| {
                if let Some(assets) = &game_assets {
                    parent.spawn((
                        Text::new("Sakura 2P"),
                        TextFont {
                            font: assets.font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                } else {
                    parent.spawn((
                        Text::new("Sakura 2P"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        });
    });
    
    println!("=== Fate/stay night Heaven's Feel ===");
    println!("Shirou Runner game started successfully!");
    println!("Click Start Game button to begin");
}

/// 处理开始按钮点击
pub fn handle_start_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>)
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8));
                next_state.set(GameState::Playing);
                println!("🎮 Starting game!");
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            }
        }
    }
}

/// 处理加载按钮点击
pub fn handle_load_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LoadButton>)
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.8));
                next_state.set(GameState::LoadTable);
                println!("📂 Opening load interface!");
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8));
            }
        }
    }
}

/// 处理角色选择按钮
pub fn handle_character_select(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CharacterSelectButton),
        Changed<Interaction>
    >,
    mut character_selection: ResMut<CharacterSelection>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                character_selection.selected_character = button.character_type.clone();
                println!("选择角色: {:?}", button.character_type);
                
                // 更新按钮颜色表示选中状态
                match button.character_type {
                    CharacterType::Shirou1 => {
                        *color = BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.8));
                    }
                    CharacterType::Shirou2 => {
                        *color = BackgroundColor(Color::srgba(0.2, 0.2, 0.5, 0.8));
                    }
                }
            }
            Interaction::Hovered => {
                match button.character_type {
                    CharacterType::Shirou1 => {
                        *color = BackgroundColor(Color::srgba(0.4, 0.15, 0.15, 0.8));
                    }
                    CharacterType::Shirou2 => {
                        *color = BackgroundColor(Color::srgba(0.15, 0.15, 0.4, 0.8));
                    }
                }
            }
            Interaction::None => {
                match button.character_type {
                    CharacterType::Shirou1 => {
                        *color = BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8));
                    }
                    CharacterType::Shirou2 => {
                        *color = BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 0.8));
                    }
                }
            }
        }
    }
}

/// 封面渐变动画系统 - 缓慢自然的渐变效果
pub fn cover_fade_animation(
    mut cover_query: Query<(&mut Sprite, &mut CoverFadeState), Or<(With<CoverImage1>, With<CoverImage2>)>>,
    time: Res<Time>,
) {
    // 使用更长的循环时间，让渐变更缓慢
    let elapsed_time = time.elapsed_secs();
    let cycle_duration = 15.0; // 15秒一个完整循环，更慢更稳定
    let cycle_progress = (elapsed_time % cycle_duration) / cycle_duration;
    
    for (mut sprite, mut fade_state) in cover_query.iter_mut() {
        // 使用更平滑的渐变函数
        let base_alpha = (cycle_progress * 2.0 * std::f32::consts::PI).sin();
        
        // 根据图片类型调整透明度
        let final_alpha = if fade_state.fade_direction > 0.0 {
            // 第一张图片：缓慢淡入淡出
            (base_alpha + 1.0) * 0.5
        } else {
            // 第二张图片：与第一张相反
            ((-base_alpha) + 1.0) * 0.5
        };
        
        // 使用更平滑的缓动函数，减少突兀感
        let eased_alpha = final_alpha * final_alpha * (3.0 - 2.0 * final_alpha); // smoothstep函数
        
        // 限制透明度变化范围，避免完全透明
        let clamped_alpha = eased_alpha.clamp(0.1, 0.9);
        
        sprite.color.set_alpha(clamped_alpha);
        fade_state.alpha = clamped_alpha;
    }
}



/// 清理菜单界面
pub fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MenuUI>>,
) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}