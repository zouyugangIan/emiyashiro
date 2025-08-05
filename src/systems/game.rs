//! 核心游戏系统
//! 
//! 包含游戏场景的设置、清理和核心游戏逻辑管理。

use bevy::prelude::*;
use crate::{
    components::*,
    states::*,
    resources::*,
};

/// 设置游戏场景
/// 
/// 初始化游戏世界，包括摄像机、地面、玩家等基本实体。
/// 根据角色选择创建对应的玩家角色。
/// 如果有加载的游戏状态，则恢复该状态。
/// 
/// # 参数
/// * `commands` - 用于生成实体的命令缓冲区
/// * `character_selection` - 当前选择的角色
/// * `game_assets` - 游戏资源句柄
/// * `camera_query` - 摄像机查询
/// * `player_query` - 玩家查询
/// * `ground_query` - 地面查询
/// * `loaded_game_state` - 加载的游戏状态
/// * `game_stats` - 游戏统计
pub fn setup_game(
    mut commands: Commands,
    mut character_selection: ResMut<CharacterSelection>,
    game_assets: Res<GameAssets>,
    camera_query: Query<Entity, With<Camera>>,
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
    mut loaded_game_state: ResMut<crate::systems::ui::LoadedGameState>,
    mut game_stats: ResMut<GameStats>,
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
            crate::systems::collision::CollisionBox::new(GameConfig::GROUND_SIZE),
        ));
    }
    
    // 只有在没有玩家时才创建玩家
    if player_query.is_empty() {
        // 根据选择的角色创建玩家
        let texture = match character_selection.selected_character {
            CharacterType::Shirou1 => game_assets.shirou1_texture.clone(),
            CharacterType::Shirou2 => game_assets.shirou2_texture.clone(),
        };
        
        println!("🎭 选择的角色: {:?}", character_selection.selected_character);
        
        // 创建带动画的角色
        let sprite_animation = crate::systems::sprite_animation::create_character_animation(&character_selection.selected_character);
        
        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_translation(GameConfig::PLAYER_START_POS)
                .with_scale(Vec3::new(0.2, 0.2, 1.0)), // 缩放图片
            Player,
            Velocity { x: 0.0, y: 0.0 },
            PlayerState::default(),
            sprite_animation,
            crate::systems::collision::CollisionBox::new(GameConfig::PLAYER_SIZE),
        ));
        
        println!("🗡️ Shirou Emiya enters the battle!");
        println!("Controls:");
        println!("  A/D or ←/→ : Move left/right");
        println!("  W or ↑     : Jump");
        println!("  S or ↓     : Crouch");
        println!("  ESC        : Pause game");
    } else {
        println!("Player already exists, continuing game");
    }
    
    // 检查是否需要恢复加载的游戏状态
    if loaded_game_state.should_restore {
        if let Some(state) = &loaded_game_state.state {
            println!("🔄 恢复加载的游戏状态");
            
            // 恢复角色选择
            character_selection.selected_character = state.selected_character.clone();
            
            // 恢复游戏统计
            game_stats.distance_traveled = state.distance_traveled;
            game_stats.jump_count = state.jump_count;
            game_stats.play_time = state.play_time;
            
            println!("   角色: {:?}", state.selected_character);
            println!("   分数: {}", state.score);
            println!("   距离: {:.1}m", state.distance_traveled);
            println!("   时间: {:.1}s", state.play_time);
            
            // 标记状态已恢复
            loaded_game_state.should_restore = false;
        }
    }
}

/// 处理游戏输入（暂停和返回菜单）
/// 
/// 使用统一的 GameInput 接口处理游戏状态切换。
/// 支持 ESC 键暂停/恢复游戏，Q 键返回主菜单。
pub fn handle_game_input(
    game_input: Res<crate::systems::input::GameInput>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    match current_state.get() {
        GameState::Playing => {
            if game_input.pause {
                next_state.set(GameState::Paused);
                println!("游戏暂停");
            }
        }
        GameState::Paused => {
            if game_input.pause {
                next_state.set(GameState::Playing);
                println!("继续游戏");
            } else if game_input.cancel {
                next_state.set(GameState::Menu);
                println!("返回主菜单");
            }
        }
        _ => {}
    }
}

/// 恢复加载的游戏状态中的实体位置
pub fn restore_loaded_game_entities(
    mut loaded_game_state: ResMut<crate::systems::ui::LoadedGameState>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut game_stats: ResMut<GameStats>,
    mut character_selection: ResMut<CharacterSelection>,
    mut audio_state_manager: ResMut<AudioStateManager>,
) {
    use crate::systems::text_constants::{StatusText, SaveLoadText};
    
    if loaded_game_state.should_restore {
        if let Some(state) = &loaded_game_state.state {
            println!("{}", StatusText::LOADING_GAME);
            
            // 恢复玩家状态
            if let Ok((mut player_transform, mut player_velocity, mut player_state)) = player_query.single_mut() {
                player_transform.translation = state.player_position;
                *player_velocity = state.player_velocity.clone();
                player_state.is_grounded = state.player_grounded;
                player_state.is_crouching = state.player_crouching;
                
                println!("🔄 Player state restored:");
                println!("   Position: ({:.1}, {:.1})", state.player_position.x, state.player_position.y);
                println!("   Animation: {}", state.player_animation_state);
                println!("   Grounded: {}", state.player_grounded);
            }
            
            // 恢复摄像机状态
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                camera_transform.translation = state.camera_position;
                println!("🔄 Camera position restored: ({:.1}, {:.1})", state.camera_position.x, state.camera_position.y);
            }
            
            // 恢复游戏统计
            game_stats.distance_traveled = state.distance_traveled;
            game_stats.jump_count = state.jump_count;
            game_stats.play_time = state.play_time;
            
            println!("🔄 Game stats restored:");
            println!("   Score: {}", state.score);
            println!("   Distance: {:.1}m", state.distance_traveled);
            println!("   Jumps: {}", state.jump_count);
            println!("   Time: {:.1}s", state.play_time);
            
            // 恢复角色选择
            character_selection.selected_character = state.selected_character.clone();
            println!("🔄 Character selection restored: {:?}", state.selected_character);
            
            // 恢复音频状态
            audio_state_manager.music_playing = state.music_playing;
            audio_state_manager.music_volume = state.audio_volume;
            
            println!("🔄 Audio state restored:");
            println!("   Music playing: {}", state.music_playing);
            println!("   Volume: {:.1}", state.audio_volume);
            
            println!("✅ {}", SaveLoadText::LOAD_SUCCESS);
            
            // 标记恢复完成
            loaded_game_state.should_restore = false;
        }
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