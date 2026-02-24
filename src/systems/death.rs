//! 死亡与复活系统

use bevy::prelude::*;

use crate::{
    components::{Health, Player, PlayerState, ShroudState, Velocity},
    events::{DamageEvent, DamageSource},
    resources::{GameAssets, GameConfig, GameStats},
    states::GameState,
};

const DEATH_ZONE_Y: f32 = -400.0;

#[derive(Component)]
pub struct GameOverUiRoot;

/// 检测玩家是否坠落到死亡区并发出伤害事件。
pub fn check_player_fall_death(
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    if let Some((player_entity, transform)) = player_query.iter().next()
        && transform.translation.y < DEATH_ZONE_Y
    {
        damage_writer.write(DamageEvent {
            target: player_entity,
            amount: f32::MAX,
            source: DamageSource::Fall,
        });
    }
}

/// 显示游戏失败界面。
pub fn setup_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_assets: Option<Res<GameAssets>>,
) {
    let font = game_assets
        .as_ref()
        .map(|assets| assets.font.clone())
        .unwrap_or_else(|| asset_server.load("fonts/FiraSans-Bold.ttf"));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            GameOverUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font: font.clone(),
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new("Press R to Revive"),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press M for Menu"),
                TextFont {
                    font,
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
        });
}

/// 处理失败状态下的输入。
pub fn handle_game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Reviving);
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        next_state.set(GameState::Menu);
    }
}

/// 执行复活流程并恢复可控状态。
pub fn revive_player(
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PlayerState,
            &mut Health,
            &mut ShroudState,
        ),
        With<Player>,
    >,
    mut game_stats: ResMut<GameStats>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some((mut transform, mut velocity, mut player_state, mut health, mut shroud)) =
        player_query.iter_mut().next()
    {
        transform.translation = GameConfig::PLAYER_START_POS;
        velocity.x = 0.0;
        velocity.y = 0.0;
        player_state.is_grounded = true;
        player_state.is_crouching = false;
        health.current = health.max;
        shroud.disable_release();

        game_stats.distance_traveled = 0.0;
        game_stats.jump_count = 0;
        game_stats.play_time = 0.0;

        next_state.set(GameState::Playing);
    }
}

/// 清理失败界面。
pub fn cleanup_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
