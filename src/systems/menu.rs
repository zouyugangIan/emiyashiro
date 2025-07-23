use bevy::prelude::*;
use crate::{
    components::*,
    states::*,
    resources::*,
};

/// 设置主菜单界面
pub fn setup_menu(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
) {
    // 创建摄像机（如果还没有的话）
    commands.spawn(Camera2d);
    
    // 如果资源已加载，创建封面背景渐变效果
    if let Some(assets) = game_assets {
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
                alpha: 1.0, // 从1.0开始，与第一张相反
                fade_direction: -1.0, // 相反方向
                fade_speed: 0.2, // 更慢的渐变速度
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
        // 游戏标题
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
                parent.spawn((
                    Text::new("开始"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
            
            // 存档按钮
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
                SaveButton,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("存档"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
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
                parent.spawn((
                    Text::new("士郎 1P"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
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
                parent.spawn((
                    Text::new("士郎 2P"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
    
    println!("=== Fate/stay night Heaven's Feel ===");
    println!("Shirou Runner 游戏启动成功！");
    println!("点击开始按钮进入游戏");
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
                println!("🎮 开始游戏！");
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

/// 封面渐变动画系统
pub fn cover_fade_animation(
    mut cover1_query: Query<(&mut Sprite, &mut CoverFadeState), (With<CoverImage1>, Without<CoverImage2>)>,
    mut cover2_query: Query<(&mut Sprite, &mut CoverFadeState), (With<CoverImage2>, Without<CoverImage1>)>,
    time: Res<Time>,
) {
    // 处理第一张封面
    if let Ok((mut sprite1, mut fade_state1)) = cover1_query.single_mut() {
        // 更新透明度 - 使用平滑的正弦函数
        fade_state1.alpha += fade_state1.fade_direction * fade_state1.fade_speed * time.delta_secs();
        
        // 限制透明度范围并使用平滑过渡
        if fade_state1.alpha <= 0.0 {
            fade_state1.alpha = 0.0;
            fade_state1.fade_direction = 1.0;
        } else if fade_state1.alpha >= 1.0 {
            fade_state1.alpha = 1.0;
            fade_state1.fade_direction = -1.0;
        }
        
        // 使用平滑的渐变曲线
        let smooth_alpha = (fade_state1.alpha * std::f32::consts::PI / 2.0).sin();
        sprite1.color.set_alpha(smooth_alpha);
    }
    
    // 处理第二张封面 - 与第一张相反的节奏
    if let Ok((mut sprite2, mut fade_state2)) = cover2_query.single_mut() {
        fade_state2.alpha += fade_state2.fade_direction * fade_state2.fade_speed * time.delta_secs();
        
        if fade_state2.alpha <= 0.0 {
            fade_state2.alpha = 0.0;
            fade_state2.fade_direction = 1.0;
        } else if fade_state2.alpha >= 1.0 {
            fade_state2.alpha = 1.0;
            fade_state2.fade_direction = -1.0;
        }
        
        // 第二张图片使用相反的透明度，创造交替效果
        let smooth_alpha = (fade_state2.alpha * std::f32::consts::PI / 2.0).sin();
        sprite2.color.set_alpha(1.0 - smooth_alpha);
    }
}

/// 处理存档按钮点击
pub fn handle_save_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SaveButton>)
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.8));
                println!("💾 存档功能 - 暂未实现");
                // TODO: 实现存档功能
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8));
            }
        }
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