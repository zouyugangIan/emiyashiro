use bevy::prelude::*;
use crate::{
    resources::*,
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

/// 设置游戏内 HUD
pub fn setup_game_hud(
    mut commands: Commands,
    _game_assets: Res<GameAssets>,
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
        parent.spawn((
            Text::new("分数: 0"),
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
        
        // 距离显示
        parent.spawn((
            Text::new("距离: 0m"),
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
            Text::new("WASD/方向键: 移动 | ESC: 菜单"),
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
        **score_text = format!("分数: {}", score);
    }
    
    // 更新距离显示
    if let Ok(mut distance_text) = distance_query.single_mut() {
        **distance_text = format!("距离: {}m", game_stats.distance_traveled as u32);
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

/// 设置暂停菜单
pub fn setup_pause_menu(
    mut commands: Commands,
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
        PauseMenu,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(200.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderColor(Color::WHITE),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("游戏暂停"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            parent.spawn((
                Text::new("按 ESC 继续游戏\n按 Q 返回主菜单"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
    });
}

/// 清理暂停菜单
pub fn cleanup_pause_menu(
    mut commands: Commands,
    pause_query: Query<Entity, With<PauseMenu>>,
) {
    for entity in pause_query.iter() {
        commands.entity(entity).despawn();
    }
}