use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::shared_utils::*;
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
    save_path: PathBuf,
    game_state: CompleteGameState,
    metadata: SaveFileMetadata,
    compression_enabled: bool,
    compression_level: u32,
) -> Result<(), SaveSystemError> {
    // 创建保存数据结构
    let save_data = SaveFileData {
        version: "1.0".to_string(),
        metadata,
        game_state,
        checksum: String::new(), // 稍后计算
    };

    // 序列化数据
    let json_data = serde_json::to_string_pretty(&save_data)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    // 计算校验和
    let checksum = calculate_checksum(json_data.as_bytes());
    let mut save_data_with_checksum = save_data;
    save_data_with_checksum.checksum = checksum;

    // 重新序列化包含校验和的数据
    let final_json_data = serde_json::to_string_pretty(&save_data_with_checksum)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    // 根据设置决定是否压缩
    let file_data = if compression_enabled {
        compress_data(final_json_data.as_bytes(), compression_level)
            .map_err(|e| SaveSystemError::CompressionFailed(e.to_string()))?
    } else {
        final_json_data.into_bytes()
    };

    // 异步写入文件
    fs::write(&save_path, file_data)
        .await
        .map_err(|e| SaveSystemError::FileWriteFailed(e.to_string()))?;

    Ok(())
}

/// 异步加载游戏状态
pub async fn load_game_state_async(
    save_path: PathBuf,
    compression_enabled: bool,
) -> Result<(CompleteGameState, SaveFileMetadata), SaveSystemError> {
    // 异步读取文件
    let file_data = fs::read(&save_path)
        .await
        .map_err(|e| SaveSystemError::FileNotFound(e.to_string()))?;

    // 根据设置决定是否解压缩
    let json_data = if compression_enabled {
        let decompressed = decompress_data(&file_data)
            .map_err(|e| SaveSystemError::DecompressionFailed(e.to_string()))?;
        String::from_utf8(decompressed)
            .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?
    } else {
        String::from_utf8(file_data)
            .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?
    };

    // 反序列化数据
    let mut save_data: SaveFileData = serde_json::from_str(&json_data)
        .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?;

    // 验证校验和
    let received_checksum = save_data.checksum.clone();
    save_data.checksum = String::new(); // 重置校验和字段以进行一致性检查

    let json_for_check = serde_json::to_string_pretty(&save_data)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    let calculated_checksum = calculate_checksum(json_for_check.as_bytes());

    if received_checksum != calculated_checksum {
        warn!(
            "Checksum mismatch! Received: {}, Calculated: {}",
            received_checksum, calculated_checksum
        );
        return Err(SaveSystemError::ChecksumMismatch);
    }

    Ok((save_data.game_state, save_data.metadata))
}

/// 异步扫描存档文件
pub async fn scan_save_files_async(
    save_directory: PathBuf,
) -> Result<Vec<SaveFileMetadata>, SaveSystemError> {
    let mut save_files = Vec::new();

    // 确保目录存在
    if !save_directory.exists() {
        fs::create_dir_all(&save_directory)
            .await
            .map_err(|e| SaveSystemError::DirectoryCreationFailed(e.to_string()))?;
        return Ok(save_files);
    }

    // 读取目录内容
    let mut entries = fs::read_dir(&save_directory)
        .await
        .map_err(|e| SaveSystemError::DirectoryReadFailed(e.to_string()))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| SaveSystemError::DirectoryReadFailed(e.to_string()))?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match load_save_metadata_async(path.clone()).await {
                Ok(metadata) => save_files.push(metadata),
                Err(e) => {
                    warn!("Failed to load metadata for {:?}: {:?}", path, e);
                    // 继续处理其他文件，不因单个文件错误而失败
                }
            }
        }
    }

    // 按保存时间排序（最新的在前）
    save_files.sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));

    Ok(save_files)
}

/// 异步加载存档元数据
async fn load_save_metadata_async(save_path: PathBuf) -> Result<SaveFileMetadata, SaveSystemError> {
    let file_data = fs::read(&save_path)
        .await
        .map_err(|e| SaveSystemError::FileNotFound(e.to_string()))?;

    // 获取文件大小（暂时不使用）
    let _file_size = file_data.len() as u64;

    // 尝试解压缩（如果需要）
    let json_data = if is_compressed(&file_data) {
        let decompressed = decompress_data(&file_data)
            .map_err(|e| SaveSystemError::DecompressionFailed(e.to_string()))?;
        String::from_utf8(decompressed)
            .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?
    } else {
        String::from_utf8(file_data)
            .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?
    };

    // 只解析元数据部分以提高性能
    let save_data: SaveFileData = serde_json::from_str(&json_data)
        .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?;

    let mut metadata = save_data.metadata;
    metadata.file_path = save_path.to_string_lossy().to_string();

    Ok(metadata)
}

/// 保存文件数据结构
#[derive(Serialize, Deserialize)]
pub struct SaveFileData {
    pub version: String,
    pub metadata: SaveFileMetadata,
    pub game_state: CompleteGameState,
    pub checksum: String,
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
