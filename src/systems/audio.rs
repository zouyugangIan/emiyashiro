//! 音频系统
//! 
//! 包含背景音乐、音效播放和音频状态管理。

use bevy::prelude::*;
use crate::resources::*;

/// 音频管理资源
/// 
/// 跟踪当前播放的音乐状态，防止重复播放。
#[derive(Resource, Default)]
pub struct AudioManager {
    pub menu_music_playing: bool,
    pub game_music_playing: bool,
}

/// 播放菜单音乐
pub fn play_menu_music(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    audio_settings: Res<AudioSettings>,
    mut audio_manager: ResMut<AudioManager>,
) {
    // 只有在资源存在且音乐未播放时才播放
    if let Some(assets) = game_assets {
        if !audio_manager.menu_music_playing && audio_settings.music_enabled {
            commands.spawn((
                AudioPlayer(assets.menu_music.clone()),
                PlaybackSettings::LOOP,
            ));
            audio_manager.menu_music_playing = true;
            println!("🎵 开始播放菜单音乐");
        }
    }
}

/// 播放游戏音乐
pub fn play_game_music(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    audio_settings: Res<AudioSettings>,
    mut audio_manager: ResMut<AudioManager>,
) {
    // 只有在资源存在且音乐未播放时才播放
    if let Some(assets) = game_assets {
        if !audio_manager.game_music_playing && audio_settings.music_enabled {
            commands.spawn((
                AudioPlayer(assets.game_music.clone()),
                PlaybackSettings::LOOP,
            ));
            audio_manager.game_music_playing = true;
            println!("🎵 开始播放游戏音乐");
        }
    }
}

/// 停止所有音乐
pub fn stop_all_music(
    mut commands: Commands,
    audio_query: Query<Entity, With<AudioPlayer>>,
    mut audio_manager: ResMut<AudioManager>,
) {
    for entity in audio_query.iter() {
        commands.entity(entity).despawn();
    }
    audio_manager.menu_music_playing = false;
    audio_manager.game_music_playing = false;
    println!("🔇 停止所有音乐");
}

/// 停止菜单音乐
pub fn stop_menu_music(
    mut commands: Commands,
    audio_query: Query<Entity, With<AudioPlayer>>,
    mut audio_manager: ResMut<AudioManager>,
) {
    if audio_manager.menu_music_playing {
        // 停止所有音频实体
        for entity in audio_query.iter() {
            commands.entity(entity).despawn();
        }
        audio_manager.menu_music_playing = false;
        println!("🔇 停止菜单音乐");
    }
}

/// 停止游戏音乐
pub fn stop_game_music(
    mut commands: Commands,
    audio_query: Query<Entity, With<AudioPlayer>>,
    mut audio_manager: ResMut<AudioManager>,
) {
    if audio_manager.game_music_playing {
        // 停止所有音频实体
        for entity in audio_query.iter() {
            commands.entity(entity).despawn();
        }
        audio_manager.game_music_playing = false;
        println!("🔇 停止游戏音乐");
    }
}

/// 播放游戏音乐（带停止菜单音乐）
pub fn play_game_music_and_stop_menu(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    audio_settings: Res<AudioSettings>,
    mut audio_manager: ResMut<AudioManager>,
    mut audio_state_manager: ResMut<AudioStateManager>,
    audio_query: Query<Entity, With<AudioPlayer>>,
) {
    // 先停止菜单音乐
    if audio_manager.menu_music_playing {
        for entity in audio_query.iter() {
            commands.entity(entity).despawn();
        }
        audio_manager.menu_music_playing = false;
        println!("🔇 停止菜单音乐");
    }
    
    // 然后播放游戏音乐
    if let Some(assets) = game_assets {
        if !audio_manager.game_music_playing && audio_settings.music_enabled {
            commands.spawn((
                AudioPlayer(assets.game_music.clone()),
                PlaybackSettings::LOOP,
            ));
            audio_manager.game_music_playing = true;
            audio_state_manager.music_playing = true;
            audio_state_manager.music_volume = audio_settings.music_volume;
            println!("🎵 开始播放游戏音乐");
        }
    }
}

/// 暂停时保持音乐播放
pub fn maintain_audio_during_pause(
    audio_state_manager: Res<AudioStateManager>,
    audio_manager: Res<AudioManager>,
    audio_query: Query<&AudioPlayer>,
) {
    // 在暂停状态下，音乐继续播放
    // 这个系统确保音频状态在暂停时不被改变
    if audio_state_manager.music_playing && audio_manager.game_music_playing {
        // 验证音频实体仍然存在
        let audio_entities_count = audio_query.iter().count();
        if audio_entities_count == 0 {
            println!("⚠️ Warning: Audio state indicates music playing but no audio entities found");
        }
        // 音乐继续播放，不做任何操作
        // Bevy的音频系统会自动处理播放状态
    }
}

/// 恢复游戏时的音频处理
pub fn resume_audio_after_pause(
    audio_state_manager: ResMut<AudioStateManager>,
    audio_manager: Res<AudioManager>,
) {
    // 从暂停恢复时，确保音频状态正确
    if audio_state_manager.music_playing && audio_manager.game_music_playing {
        println!("🎵 Music continues seamlessly after pause");
    } else if audio_state_manager.music_playing && !audio_manager.game_music_playing {
        println!("⚠️ Audio state mismatch detected - music should be playing");
        // 可以在这里添加音频恢复逻辑
    }
}

/// 保存音频状态到游戏状态
pub fn capture_audio_state(
    audio_manager: Res<AudioManager>,
    audio_state_manager: Res<AudioStateManager>,
    audio_settings: Res<AudioSettings>,
) -> AudioState {
    AudioState {
        music_playing: audio_manager.game_music_playing,
        music_volume: audio_settings.music_volume,
        sfx_volume: audio_settings.sfx_volume,
        master_volume: audio_settings.master_volume,
        music_enabled: audio_settings.music_enabled,
        // TODO: 实现音频位置跟踪
        music_position: 0.0,
    }
}

/// 从游戏状态恢复音频状态
pub fn restore_audio_state(
    mut commands: Commands,
    audio_state: &AudioState,
    game_assets: Option<Res<GameAssets>>,
    mut audio_manager: ResMut<AudioManager>,
    mut audio_state_manager: ResMut<AudioStateManager>,
    mut audio_settings: ResMut<AudioSettings>,
    audio_query: Query<Entity, With<AudioPlayer>>,
) {
    // 更新音频设置
    audio_settings.music_volume = audio_state.music_volume;
    audio_settings.sfx_volume = audio_state.sfx_volume;
    audio_settings.master_volume = audio_state.master_volume;
    audio_settings.music_enabled = audio_state.music_enabled;
    
    // 更新音频状态管理器
    audio_state_manager.music_playing = audio_state.music_playing;
    audio_state_manager.music_volume = audio_state.music_volume;
    
    // 如果需要播放音乐但当前没有播放
    if audio_state.music_playing && !audio_manager.game_music_playing && audio_state.music_enabled {
        if let Some(assets) = game_assets {
            // 停止现有音频
            for entity in audio_query.iter() {
                commands.entity(entity).despawn();
            }
            
            // 开始播放游戏音乐
            commands.spawn((
                AudioPlayer(assets.game_music.clone()),
                PlaybackSettings::LOOP,
            ));
            
            audio_manager.game_music_playing = true;
            println!("🎵 Audio state restored - game music playing");
        }
    }
    // 如果不需要播放音乐但当前在播放
    else if !audio_state.music_playing && audio_manager.game_music_playing {
        // 停止音乐
        for entity in audio_query.iter() {
            commands.entity(entity).despawn();
        }
        audio_manager.game_music_playing = false;
        println!("🔇 Audio state restored - music stopped");
    }
    
    println!("🔊 Audio state fully restored:");
    println!("   Music playing: {}", audio_state.music_playing);
    println!("   Music volume: {:.1}", audio_state.music_volume);
    println!("   Music enabled: {}", audio_state.music_enabled);
}

/// 音频状态结构
#[derive(Clone, Debug)]
pub struct AudioState {
    pub music_playing: bool,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub master_volume: f32,
    pub music_enabled: bool,
    pub music_position: f32, // 未来实现
}