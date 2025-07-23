use bevy::prelude::*;
use bevy::audio::Volume;
use crate::resources::*;

/// 音频管理资源
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
                PlaybackSettings::LOOP.with_volume(Volume::Linear(audio_settings.music_volume)),
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
                PlaybackSettings::LOOP.with_volume(Volume::Linear(audio_settings.music_volume)),
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
    mut audio_manager: ResMut<AudioManager>,
) {
    audio_manager.menu_music_playing = false;
    println!("🔇 停止菜单音乐");
}

/// 停止游戏音乐
pub fn stop_game_music(
    mut audio_manager: ResMut<AudioManager>,
) {
    audio_manager.game_music_playing = false;
    println!("🔇 停止游戏音乐");
}