//! 主菜单系统
//!
//! 包含主菜单界面的创建、交互处理和动画效果。

use crate::{
    components::*, resources::*, states::*, systems::settings_ui::MenuSettingsButton,
    systems::ui::LoadButton,
};
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use bevy::window::PrimaryWindow;

type StartButtonInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<StartButton>),
>;

type LoadButtonInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<LoadButton>),
>;

type MenuSettingsButtonInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<MenuSettingsButton>),
>;

type CoverFadeQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut BackgroundColor,
        &'static mut ImageNode,
        &'static mut CoverFadeState,
    ),
    Or<(With<CoverImage1>, With<CoverImage2>)>,
>;

type CoverLayoutQuery<'w, 's> =
    Query<'w, 's, &'static mut ImageNode, Or<(With<CoverImage1>, With<CoverImage2>)>>;

fn fullscreen_cover_image(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

/// Returns the centered source rectangle that fills `viewport_size` without
/// distorting the image. Excess pixels are cropped like CSS `object-fit: cover`.
pub fn cover_source_rect(source_size: Vec2, viewport_size: Vec2) -> Option<Rect> {
    if source_size.min_element() <= 0.0 || viewport_size.min_element() <= 0.0 {
        return None;
    }

    let source_aspect = source_size.x / source_size.y;
    let viewport_aspect = viewport_size.x / viewport_size.y;
    let crop_size = if source_aspect > viewport_aspect {
        Vec2::new(source_size.y * viewport_aspect, source_size.y)
    } else {
        Vec2::new(source_size.x, source_size.x / viewport_aspect)
    };
    let crop_min = (source_size - crop_size) * 0.5;
    Some(Rect::from_corners(crop_min, crop_min + crop_size))
}

/// Keeps both fading menu covers flush with the current window on resize.
pub fn update_menu_cover_layout(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut cover_query: CoverLayoutQuery,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };
    let viewport_size = Vec2::new(window.width(), window.height());

    for mut image_node in &mut cover_query {
        image_node.image_mode = NodeImageMode::Stretch;
        if let Some(image) = images.get(&image_node.image) {
            image_node.rect = cover_source_rect(image.size_f32(), viewport_size);
        }
    }
}

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

    // 创建封面背景 - 即使资源未加载也创建占位符
    // 第一张封面图片 - 使用UI节点实现响应式布局
    let cover1_image = game_assets
        .as_ref()
        .map(|assets| assets.get_current_cover())
        .unwrap_or_default();

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        fullscreen_cover_image(cover1_image),
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
        ZIndex(0),
        MenuUI,
        CoverImage1,
        CoverFadeState::default(),
    ));

    // 第二张封面图片 - 使用UI节点实现响应式布局，从透明开始
    let cover2_image = if let Some(ref assets) = game_assets {
        let next_cover_index = (assets.current_cover_index + 1) % assets.cover_textures.len();
        assets.cover_textures[next_cover_index].clone()
    } else {
        Handle::default()
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        fullscreen_cover_image(cover2_image),
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)), // 从透明开始
        ZIndex(1),
        MenuUI,
        CoverImage2,
        CoverFadeState {
            alpha: 0.0,           // 从0.0开始
            fade_direction: -1.0, // 负方向表示第二张图片
        },
    ));

    // 创建UI根节点 - 确保在封面图片之上
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex(2), // 确保在封面图片之上
            MenuUI,
        ))
        .with_children(|parent| {
            // 游戏标题 - 使用英文文本常量
            if let Some(assets) = &game_assets {
                parent.spawn((
                    Text::new(crate::systems::text_constants::MainMenuText::TITLE),
                    TextFont {
                        font: assets.font.clone().into(),
                        font_size: FontSize::Px(48.0),
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
                    Text::new(crate::systems::text_constants::MainMenuText::TITLE),
                    TextFont {
                        font_size: FontSize::Px(48.0),
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
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|parent| {
                    // 开始按钮
                    parent
                        .spawn((
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
                            BorderColor::all(Color::WHITE),
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                            StartButton,
                        ))
                        .with_children(|parent| {
                            if let Some(assets) = &game_assets {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::START_GAME,
                                    ),
                                    TextFont {
                                        font: assets.font.clone().into(),
                                        font_size: FontSize::Px(24.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            } else {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::START_GAME,
                                    ),
                                    TextFont {
                                        font_size: FontSize::Px(24.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            }
                        });

                    // 加载存档按钮
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
                            BorderColor::all(Color::WHITE),
                            BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8)),
                            LoadButton,
                        ))
                        .with_children(|parent| {
                            if let Some(assets) = &game_assets {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::LOAD_GAME,
                                    ),
                                    TextFont {
                                        font: assets.font.clone().into(),
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            } else {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::LOAD_GAME,
                                    ),
                                    TextFont {
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            }
                        });

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
                            BorderColor::all(Color::srgba(0.75, 0.62, 0.38, 0.9)),
                            BackgroundColor(Color::srgba(0.14, 0.12, 0.10, 0.85)),
                            MenuSettingsButton,
                        ))
                        .with_children(|parent| {
                            if let Some(assets) = &game_assets {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::SETTINGS,
                                    ),
                                    TextFont {
                                        font: assets.font.clone().into(),
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.92, 0.88, 0.82, 1.0)),
                                ));
                            } else {
                                parent.spawn((
                                    Text::new(
                                        crate::systems::text_constants::MainMenuText::SETTINGS,
                                    ),
                                    TextFont {
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.92, 0.88, 0.82, 1.0)),
                                ));
                            }
                        });
                });

            // 角色选择按钮
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|parent| {
                    // 角色1按钮
                    parent
                        .spawn((
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
                            BorderColor::all(Color::WHITE),
                            BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8)),
                            CharacterSelectButton {
                                character_type: CharacterType::Shirou,
                            },
                        ))
                        .with_children(|parent| {
                            if let Some(assets) = &game_assets {
                                parent.spawn((
                                    Text::new("Shirou 1P"),
                                    TextFont {
                                        font: assets.font.clone().into(),
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            } else {
                                parent.spawn((
                                    Text::new("Shirou 1P"),
                                    TextFont {
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            }
                        });

                    // 角色2按钮
                    parent
                        .spawn((
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
                            BorderColor::all(Color::WHITE),
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 0.8)),
                            CharacterSelectButton {
                                character_type: CharacterType::Sakura,
                            },
                        ))
                        .with_children(|parent| {
                            if let Some(assets) = &game_assets {
                                parent.spawn((
                                    Text::new("Sakura 2P"),
                                    TextFont {
                                        font: assets.font.clone().into(),
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            } else {
                                parent.spawn((
                                    Text::new("Sakura 2P"),
                                    TextFont {
                                        font_size: FontSize::Px(18.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            }
                        });
                });
        });

    crate::debug_log!("=== Fate/stay night Heaven's Feel ===");
    crate::debug_log!("Shirou Runner game started successfully!");
    crate::debug_log!("Click Start Game button to begin");
}

/// 处理开始按钮点击
pub fn handle_start_button(
    mut interaction_query: StartButtonInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    mut loaded_game_state: ResMut<crate::systems::ui::LoadedGameState>,
    mut save_load_ui_state: Option<ResMut<crate::systems::ui::SaveLoadUiState>>,
    mut game_stats: ResMut<GameStats>,
    mut pause_manager: ResMut<PauseManager>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8));

                // 重置所有游戏状态，确保从头开始
                loaded_game_state.state = None;
                loaded_game_state.should_restore = false;

                // 重置游戏统计
                game_stats.distance_traveled = 0.0;
                game_stats.jump_count = 0;
                game_stats.play_time = 0.0;

                // 清理暂停管理器状态
                pause_manager.clear_pause_state();

                // 清理存档UI运行状态
                if let Some(ref mut save_ui) = save_load_ui_state {
                    save_ui.is_busy = false;
                    save_ui.pending_load_index = None;
                    save_ui.status_message.clear();
                    save_ui.error_message.clear();
                }

                NextState::set_if_neq(&mut next_state, GameState::Playing);
                crate::debug_log!("🎮 Starting NEW game! (All states reset)");
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

/// 打开设置浮层（主菜单）
pub fn handle_menu_settings_button(
    mut commands: Commands,
    mut interaction_query: MenuSettingsButtonInteractionQuery,
    game_assets: Option<Res<GameAssets>>,
    audio_settings: Option<Res<AudioSettings>>,
    existing_settings: Query<(), With<crate::systems::settings_ui::SettingsOverlayRoot>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                crate::systems::settings_ui::open_settings_overlay_from_resources(
                    &mut commands,
                    game_assets.as_deref(),
                    audio_settings.as_deref(),
                    &existing_settings,
                );
                *color = BackgroundColor(Color::srgba(0.22, 0.18, 0.14, 0.95));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.20, 0.17, 0.14, 0.92));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.14, 0.12, 0.10, 0.85));
            }
        }
    }
}

/// 处理加载按钮点击
pub fn handle_load_button(
    mut interaction_query: LoadButtonInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    save_file_manager: Res<SaveFileManager>,
    mut loaded_game_state: ResMut<crate::systems::ui::LoadedGameState>,
    mut save_load_ui_state: Option<ResMut<crate::systems::ui::SaveLoadUiState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.8));

                crate::debug_log!("📂 Opening load interface from main menu");
                crate::debug_log!("   Available saves: {}", save_file_manager.save_files.len());

                // 记录来源状态
                loaded_game_state.previous_state = Some(GameState::Menu);
                if let Some(ref mut save_ui) = save_load_ui_state {
                    save_ui.pending_load_index = None;
                    save_ui.error_message.clear();
                    if !save_ui.is_busy {
                        save_ui.status_message.clear();
                    }
                }
                NextState::set_if_neq(&mut next_state, GameState::LoadTable);
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
        Changed<Interaction>,
    >,
    mut character_selection: ResMut<CharacterSelection>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                character_selection.selected_character = button.character_type.clone();
                crate::debug_log!("选择角色: {:?}", button.character_type);

                // 更新按钮颜色表示选中状态
                match button.character_type {
                    CharacterType::Shirou => {
                        *color = BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.8));
                    }
                    CharacterType::Sakura => {
                        *color = BackgroundColor(Color::srgba(0.2, 0.2, 0.5, 0.8));
                    }
                }
            }
            Interaction::Hovered => match button.character_type {
                CharacterType::Shirou => {
                    *color = BackgroundColor(Color::srgba(0.4, 0.15, 0.15, 0.8));
                }
                CharacterType::Sakura => {
                    *color = BackgroundColor(Color::srgba(0.15, 0.15, 0.4, 0.8));
                }
            },
            Interaction::None => match button.character_type {
                CharacterType::Shirou => {
                    *color = BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.8));
                }
                CharacterType::Sakura => {
                    *color = BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 0.8));
                }
            },
        }
    }
}

/// 封面渐变动画系统 - 优雅的淡入淡出效果
///
/// 实现原理：
/// - 两张图片层叠显示，通过调整透明度实现淡入淡出
/// - 当第一张图片完全淡出（alpha=0.1）时，切换其内容为下下张图片
/// - 当第二张图片完全淡出时，切换其内容为下下张图片
/// - 这样始终保持两张不同的图片在淡入淡出
pub fn cover_fade_animation(
    mut game_assets: Option<ResMut<GameAssets>>,
    mut cover_query: CoverFadeQuery,
    time: Res<Time>,
    mut initialized: Local<bool>,
) {
    // 如果资源未加载，跳过
    let Some(ref mut assets) = game_assets else {
        return;
    };

    // 首次初始化时立即加载图片
    if !*initialized {
        for (mut background_color, mut image_node, fade_state) in cover_query.iter_mut() {
            if fade_state.fade_direction > 0.0 {
                // 第一张图片：当前封面，初始完全不透明
                image_node.image = assets.get_current_cover();
                background_color.0.set_alpha(0.9);
            } else {
                // 第二张图片：下一张封面，初始完全透明
                let next_index = (assets.current_cover_index + 1) % assets.cover_textures.len();
                image_node.image = assets.cover_textures[next_index].clone();
                background_color.0.set_alpha(0.1);
            }
        }
        *initialized = true;
        crate::debug_log!(
            "🖼️ 初始化封面图片: 当前={}, 下一张={} (共{}张)",
            assets.current_cover_index,
            (assets.current_cover_index + 1) % assets.cover_textures.len(),
            assets.cover_textures.len()
        );
    }

    let elapsed_time = time.elapsed_secs();
    let cycle_duration = 5.0;
    let fade_duration = 1.0;

    let time_in_cycle = elapsed_time % cycle_duration;
    let current_cycle = (elapsed_time / cycle_duration).floor() as i32;
    let last_cycle = ((elapsed_time - time.delta_secs()) / cycle_duration).floor() as i32;

    if current_cycle > last_cycle {
        assets.next_cover();
        crate::debug_log!(
            "🖼️ 切换封面: {} (共{}张)",
            assets.current_cover_index,
            assets.cover_textures.len()
        );
    }

    let next_idx = (assets.current_cover_index + 1) % assets.cover_textures.len();

    for (mut background_color, mut image_node, mut fade_state) in cover_query.iter_mut() {
        let is_cover1 = fade_state.fade_direction > 0.0;

        if is_cover1 {
            image_node.image = assets.get_current_cover();
        } else {
            image_node.image = assets.cover_textures[next_idx].clone();
        }

        let alpha = if time_in_cycle < fade_duration {
            let t = time_in_cycle / fade_duration;
            if is_cover1 { t } else { 1.0 - t }
        } else if time_in_cycle < cycle_duration - fade_duration {
            if is_cover1 { 1.0 } else { 0.0 }
        } else {
            let t = (time_in_cycle - (cycle_duration - fade_duration)) / fade_duration;
            if is_cover1 { 1.0 - t } else { t }
        };

        background_color.0.set_alpha(alpha);
        fade_state.alpha = alpha;
    }
}

/// 清理菜单界面
pub fn cleanup_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuUI>>) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}
