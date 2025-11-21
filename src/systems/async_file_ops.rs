use bevy::prelude::*;
use std::path::PathBuf;

#[cfg(feature = "server")]
use crate::systems::server_file_ops;

use crate::{
    asset_paths,
    resources::{CompleteGameState, SaveFileMetadata},
    systems::error_handling::SaveSystemError,
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
            compression_level: 6, // 平衡压缩率和速度
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
    _save_path: PathBuf,
    _game_state: CompleteGameState,
    _metadata: SaveFileMetadata,
    _compression_enabled: bool,
    _compression_level: u32,
) -> Result<(), SaveSystemError> {
    #[cfg(feature = "server")]
    {
        server_file_ops::save_game_state_internal(
            _save_path,
            _game_state,
            _metadata,
            _compression_enabled,
            _compression_level,
        )
        .await
    }

    #[cfg(not(feature = "server"))]
    {
        // Client cannot save to disk directly
        Err(SaveSystemError::FileWriteFailed("Client cannot write to disk".into()))
    }
}

/// 异步加载游戏状态
pub async fn load_game_state_async(
    _save_path: PathBuf,
    _compression_enabled: bool,
) -> Result<(CompleteGameState, SaveFileMetadata), SaveSystemError> {
    #[cfg(feature = "server")]
    {
        server_file_ops::load_game_state_internal(_save_path, _compression_enabled).await
    }

    #[cfg(not(feature = "server"))]
    {
        Err(SaveSystemError::FileNotFound("Client cannot read from disk".into()))
    }
}

/// 异步扫描存档文件
pub async fn scan_save_files_async(
    save_directory: PathBuf,
) -> Result<Vec<SaveFileMetadata>, SaveSystemError> {
    #[cfg(feature = "server")]
    {
        server_file_ops::scan_save_files_internal(save_directory).await
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(Vec::new())
    }
}

/// 系统：更新操作进度
pub fn update_operation_progress(mut progress: ResMut<OperationProgress>, time: Res<Time>) {
    if progress.is_active {
        // 模拟进度更新（实际实现中会从异步任务获取真实进度）
        let delta = time.delta_secs() * 10.0; // 假设每秒增加10%
        progress.progress_percentage = (progress.progress_percentage + delta).min(100.0);

        if progress.progress_percentage >= 100.0 {
            progress.complete_operation();
        }
    }
}

/// 系统：显示进度指示器UI
pub fn display_progress_indicator(
    mut commands: Commands,
    progress: Res<OperationProgress>,
    asset_server: Res<AssetServer>,
    mut query: Query<Entity, With<ProgressIndicator>>,
) {
    // 清理现有的进度指示器
    for entity in query.iter_mut() {
        commands.entity(entity).despawn();
    }

    if progress.is_active {
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
                // 操作名称
                if let Some(ref operation) = progress.current_operation {
                    parent.spawn((
                        Text::new(operation.clone()),
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
                    ));
                }

                // 进度条背景
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
                        // 进度条填充
                        parent.spawn((
                            Node {
                                width: Val::Px(280.0 * progress.progress_percentage / 100.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                        ));
                    });

                // 进度百分比文本
                parent.spawn((
                    Text::new(format!("{:.1}%", progress.progress_percentage)),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
    }
}

/// 进度指示器组件标记
#[derive(Component)]
pub struct ProgressIndicator;
