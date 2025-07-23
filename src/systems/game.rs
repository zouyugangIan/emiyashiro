use bevy::prelude::*;
use crate::{
    components::*,
    states::*,
    resources::*,
};

/// 设置游戏场景
pub fn setup_game(
    mut commands: Commands,
    character_selection: Res<CharacterSelection>,
    game_assets: Res<GameAssets>,
    camera_query: Query<Entity, With<Camera>>,
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
) {
    // 确保有摄像机存在
    if camera_query.is_empty() {
        commands.spawn(Camera2d);
        println!("创建游戏摄像机");
    }
    
    // 只有在没有地面时才创建地面
    if ground_query.is_empty() {
        commands.spawn((
            Sprite {
                color: GameConfig::GROUND_COLOR,
                custom_size: Some(GameConfig::GROUND_SIZE),
                ..default()
            },
            Transform::from_translation(GameConfig::GROUND_POS),
            Ground,
        ));
    }
    
    // 只有在没有玩家时才创建玩家
    if player_query.is_empty() {
        // 根据选择的角色创建玩家
        let texture = match character_selection.selected_character {
            CharacterType::Shirou1 => game_assets.shirou1_texture.clone(),
            CharacterType::Shirou2 => game_assets.shirou2_texture.clone(),
        };
        
        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_translation(GameConfig::PLAYER_START_POS)
                .with_scale(Vec3::new(0.2, 0.2, 1.0)), // 缩放图片
            Player,
            Velocity { x: 0.0, y: 0.0 },
            PlayerState::default(),
        ));
        
        println!("🗡️ 卫宫士郎登场！");
        println!("操作说明：");
        println!("  A/D 或 ←/→ : 左右移动");
        println!("  W 或 ↑     : 跳跃");
        println!("  S 或 ↓     : 趴下");
        println!("  ESC        : 返回菜单");
    } else {
        println!("玩家已存在，继续游戏");
    }
}

/// 处理游戏输入（暂停和返回菜单）
pub fn handle_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    match current_state.get() {
        GameState::Playing => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
                println!("游戏暂停");
            }
        }
        GameState::Paused => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
                println!("继续游戏");
            } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
                next_state.set(GameState::Menu);
                println!("返回主菜单");
            }
        }
        _ => {}
    }
}

/// 清理游戏场景
pub fn cleanup_game(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
) {
    // 清理所有玩家实体
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
        println!("清理玩家实体");
    }
    
    // 清理所有地面实体
    for entity in ground_query.iter() {
        commands.entity(entity).despawn();
        println!("清理地面实体");
    }
}