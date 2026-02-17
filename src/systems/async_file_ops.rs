use bevy::prelude::*;
use std::path::PathBuf;

use crate::{
    asset_paths,
    resources::{CompleteGameState, SaveFileMetadata},
    systems::{error_handling::SaveSystemError, server_file_ops},
};

/// 异步文件操作管理器
#[derive(Resource, Default)]
pub struct AsyncFileManager {
    pub compression_enabled: bool,
    pub compression_level: u32,
    pub max_concurrent_operations: usize,
    pub operation_timeout_seconds: u64,
}

impl AsyncFileManager {
    pub fn new() -> Self {
        Self {
            compression_enabled: true,
            compression_level: 3, // Zstd level 3: good speed/ratio balance
            max_concurrent_operations: 4,
            operation_timeout_seconds: 30,
        }
    }
}

/// 进度指示器资源
#[derive(Resource, Default)]
pub struct OperationProgress {
    pub current_operation: Option<String>,
    pub progress_percentage: f32,
    pub is_active: bool,
    pub estimated_time_remaining: f32,
}

impl OperationProgress {
    pub fn start_operation(&mut self, operation_name: String) {
        self.current_operation = Some(operation_name);
        self.progress_percentage = 0.0;
        self.is_active = true;
        self.estimated_time_remaining = 0.0;
    }

    pub fn update_progress(&mut self, percentage: f32, estimated_time: f32) {
        self.progress_percentage = percentage.clamp(0.0, 100.0);
        self.estimated_time_remaining = estimated_time;
    }

    pub fn complete_operation(&mut self) {
        self.current_operation = None;
        self.progress_percentage = 100.0;
        self.is_active = false;
        self.estimated_time_remaining = 0.0;
    }
}

/// 异步保存游戏状态
pub async fn save_game_state_async(
    save_path: PathBuf,
    game_state: CompleteGameState,
    metadata: SaveFileMetadata,
    compression_enabled: bool,
    compression_level: u32,
) -> Result<(), SaveSystemError> {
    server_file_ops::save_game_state_internal(
        save_path,
        game_state,
        metadata,
        compression_enabled,
        compression_level,
    )
    .await
}

/// 异步加载游戏状态
pub async fn load_game_state_async(
    save_path: PathBuf,
    compression_enabled: bool,
) -> Result<(CompleteGameState, SaveFileMetadata), SaveSystemError> {
    server_file_ops::load_game_state_internal(save_path, compression_enabled).await
}

/// 异步扫描存档文件
pub async fn scan_save_files_async(
    save_directory: PathBuf,
) -> Result<Vec<SaveFileMetadata>, SaveSystemError> {
    server_file_ops::scan_save_files_internal(save_directory).await
}

/// 系统：更新操作进度
pub fn update_operation_progress(mut progress: ResMut<OperationProgress>, time: Res<Time>) {
    if progress.is_active {
        // 模拟进度更新（实际完成由异步任务完成回调触发）
        let delta = time.delta_secs() * 12.0;
        progress.progress_percentage = (progress.progress_percentage + delta).min(95.0);
    }
}

/// 系统：显示进度指示器UI
pub fn display_progress_indicator(
    mut commands: Commands,
    progress: Res<OperationProgress>,
    asset_server: Res<AssetServer>,
    root_query: Query<Entity, With<ProgressIndicator>>,
    mut operation_text_query: Query<
        &mut Text,
        (With<ProgressOperationText>, Without<ProgressPercentText>),
    >,
    mut percent_text_query: Query<
        &mut Text,
        (With<ProgressPercentText>, Without<ProgressOperationText>),
    >,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
) {
    if !progress.is_active {
        for entity in root_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    if root_query.is_empty() {
        let font = asset_server.load(asset_paths::FONT_FIRA_SANS);

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(50.0),
                    width: Val::Px(300.0),
                    height: Val::Px(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ProgressIndicator,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(progress.current_operation.clone().unwrap_or_default()),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ProgressOperationText,
                ));

                parent
                    .spawn((
                        Node {
                            width: Val::Px(280.0),
                            height: Val::Px(20.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(280.0 * progress.progress_percentage / 100.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                            ProgressBarFill,
                        ));
                    });

                parent.spawn((
                    Text::new(format!("{:.1}%", progress.progress_percentage)),
                    TextFont {
                        font,
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ProgressPercentText,
                ));
            });
        return;
    }

    if let Ok(mut operation_text) = operation_text_query.single_mut() {
        **operation_text = progress.current_operation.clone().unwrap_or_default();
    }

    if let Ok(mut percent_text) = percent_text_query.single_mut() {
        **percent_text = format!("{:.1}%", progress.progress_percentage);
    }

    if let Ok(mut fill_node) = fill_query.single_mut() {
        fill_node.width = Val::Px(280.0 * progress.progress_percentage / 100.0);
    }
}

/// 进度指示器组件标记
#[derive(Component)]
pub struct ProgressIndicator;

#[derive(Component)]
pub struct ProgressOperationText;

#[derive(Component)]
pub struct ProgressPercentText;

#[derive(Component)]
pub struct ProgressBarFill;
