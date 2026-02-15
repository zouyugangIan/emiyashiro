use bevy::prelude::*;

use std::fs;
use std::path::PathBuf;

use super::shared_utils::*;
use crate::{
    resources::{CompleteGameState, SaveFileData, SaveFileMetadata},
    systems::error_handling::SaveSystemError,
};

fn checksum_mismatch_is_fatal() -> bool {
    std::env::var("EMIYASHIRO_STRICT_CHECKSUM")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// 内部实现：异步保存游戏状态
pub async fn save_game_state_internal(
    save_path: PathBuf,
    game_state: CompleteGameState,
    metadata: SaveFileMetadata,
    compression_enabled: bool,
    compression_level: u32,
) -> Result<(), SaveSystemError> {
    if let Some(parent_dir) = save_path.parent() {
        fs::create_dir_all(parent_dir)
            .map_err(|e| SaveSystemError::DirectoryCreationFailed(e.to_string()))?;
    }

    // 创建保存数据结构（带校验和和算法信息）
    let save_data = SaveFileData::new(metadata, game_state);

    // 序列化数据
    let final_json_data = serde_json::to_string_pretty(&save_data)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    // 根据设置决定是否压缩
    let file_data = if compression_enabled {
        compress_data(final_json_data.as_bytes(), compression_level)
            .map_err(|e| SaveSystemError::CompressionFailed(e.to_string()))?
    } else {
        final_json_data.into_bytes()
    };

    // 原子写入文件
    atomic_write_file(&save_path, &file_data)
        .map_err(|e| SaveSystemError::FileWriteFailed(e.to_string()))?;

    Ok(())
}

/// 内部实现：异步加载游戏状态
pub async fn load_game_state_internal(
    save_path: PathBuf,
    _compression_enabled: bool,
) -> Result<(CompleteGameState, SaveFileMetadata), SaveSystemError> {
    // 读取文件
    let file_data =
        fs::read(&save_path).map_err(|e| SaveSystemError::FileNotFound(e.to_string()))?;

    // 自动识别压缩格式并解码
    let json_data = decode_file_payload(&file_data)
        .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?;

    // 优先尝试新格式
    if let Ok(save_data) = serde_json::from_str::<SaveFileData>(&json_data) {
        if !save_data.verify_checksum() {
            if checksum_mismatch_is_fatal() {
                return Err(SaveSystemError::ChecksumMismatch);
            }

            warn!(
                "Checksum mismatch for {:?}, loading in compatibility mode",
                save_path
            );
        }

        let mut metadata = save_data.metadata;
        metadata.file_path = save_path.to_string_lossy().to_string();

        return Ok((save_data.game_state, metadata));
    }

    // 兼容旧格式（仅包含 CompleteGameState）
    if let Ok(state) = serde_json::from_str::<CompleteGameState>(&json_data) {
        let metadata = SaveFileMetadata {
            name: save_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("LegacySave")
                .to_string(),
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: save_path.to_string_lossy().to_string(),
        };

        return Ok((state, metadata));
    }

    Err(SaveSystemError::DeserializationFailed(
        "Unsupported save file format".to_string(),
    ))
}

/// 内部实现：异步扫描存档文件
pub async fn scan_save_files_internal(
    save_directory: PathBuf,
) -> Result<Vec<SaveFileMetadata>, SaveSystemError> {
    let mut save_files = Vec::new();

    // 确保目录存在
    if !save_directory.exists() {
        fs::create_dir_all(&save_directory)
            .map_err(|e| SaveSystemError::DirectoryCreationFailed(e.to_string()))?;
        return Ok(save_files);
    }

    // 读取目录内容
    let entries = fs::read_dir(&save_directory)
        .map_err(|e| SaveSystemError::DirectoryReadFailed(e.to_string()))?;

    for entry in entries.flatten() {
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
async fn load_save_metadata_internal(
    save_path: PathBuf,
) -> Result<SaveFileMetadata, SaveSystemError> {
    let file_data =
        fs::read(&save_path).map_err(|e| SaveSystemError::FileNotFound(e.to_string()))?;

    let json_data = decode_file_payload(&file_data)
        .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?;

    // 新格式元数据
    if let Ok(save_data) = serde_json::from_str::<SaveFileData>(&json_data) {
        if !save_data.verify_checksum() {
            warn!("Checksum mismatch in metadata scan for {:?}", save_path);
        }

        let mut metadata = save_data.metadata;
        metadata.file_path = save_path.to_string_lossy().to_string();
        return Ok(metadata);
    }

    // 旧格式回退
    if let Ok(state) = serde_json::from_str::<CompleteGameState>(&json_data) {
        return Ok(SaveFileMetadata {
            name: save_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("LegacySave")
                .to_string(),
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: save_path.to_string_lossy().to_string(),
        });
    }

    Err(SaveSystemError::DeserializationFailed(
        "Unsupported save metadata format".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_compatibility_mode_accepts_checksum_mismatch() {
        let temp_path = std::env::temp_dir().join(format!(
            "emiyashiro-load-test-{}.json",
            uuid::Uuid::new_v4()
        ));

        let metadata = SaveFileMetadata {
            name: "compat-check".to_string(),
            score: 123,
            distance: 45.0,
            play_time: 6.0,
            save_timestamp: chrono::Utc::now(),
            file_path: "stale/path.json".to_string(),
        };

        let mut save_data = SaveFileData::new(metadata, CompleteGameState::default());
        save_data.checksum = "broken-checksum".to_string();

        let json = serde_json::to_string_pretty(&save_data).expect("serialize test save");
        fs::write(&temp_path, json.as_bytes()).expect("write test save file");

        if checksum_mismatch_is_fatal() {
            let _ = fs::remove_file(temp_path);
            return;
        }

        let result =
            futures_lite::future::block_on(load_game_state_internal(temp_path.clone(), false))
                .expect("compatibility load should succeed");

        assert_eq!(
            result.1.file_path,
            temp_path.to_string_lossy().to_string(),
            "load should always return actual file path"
        );

        let _ = fs::remove_file(temp_path);
    }
}
