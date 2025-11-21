use bevy::prelude::*;

use std::path::PathBuf;
use tokio::fs;

use super::shared_utils::*;
use crate::{
    resources::{CompleteGameState, SaveFileData, SaveFileMetadata},
    systems::error_handling::SaveSystemError,
};

/// 内部实现：异步保存游戏状态
pub async fn save_game_state_internal(
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

/// 内部实现：异步加载游戏状态
pub async fn load_game_state_internal(
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

/// 内部实现：异步扫描存档文件
pub async fn scan_save_files_internal(
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
            match load_save_metadata_internal(path.clone()).await {
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

/// 内部实现：异步加载存档元数据
async fn load_save_metadata_internal(save_path: PathBuf) -> Result<SaveFileMetadata, SaveSystemError> {
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
