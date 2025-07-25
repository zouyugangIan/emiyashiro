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
) {
    // 在暂停状态下，音乐继续播放
    // 这个系统确保音频状态在暂停时不被改变
    if audio_state_manager.music_playing {
        // 音乐继续播放，不做任何操作
        // Bevy的音频系统会自动处理播放状态
    }
}

/// 恢复游戏时的音频处理
pub fn resume_audio_after_pause(
    mut audio_state_manager: ResMut<AudioStateManager>,
) {
    // 从暂停恢复时，确保音频状态正确
    if audio_state_manager.music_playing {
        println!("🎵 音乐继续播放");
    }
}