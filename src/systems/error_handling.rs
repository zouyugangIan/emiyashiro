//! 统一错误处理系统
//!
//! 提供全面的错误处理、恢复机制和用户友好的错误消息

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// 存档系统错误类型
#[derive(Debug, Clone)]
pub enum SaveSystemError {
    // 文件操作错误
    FileNotFound(String),
    PermissionDenied(String),
    DiskSpaceInsufficient,
    FileCorrupted(String),
    DirectoryCreationFailed(String),
    FileWriteFailed(String),
    DirectoryReadFailed(String),

    // 序列化错误
    SerializationFailed(String),
    DeserializationFailed(String),

    // 压缩错误
    CompressionFailed(String),
    DecompressionFailed(String),

    // 验证错误
    InvalidFileName(String),
    ChecksumMismatch,
    VersionMismatch(String),
    NameAlreadyExists(String),

    // 输入错误
    TextInputFailed(String),
    KeyboardInputError(String),

    // 系统错误
    ResourceNotFound(String),
    StateTransitionError(String),
    AudioSystemError(String),
}

impl SaveSystemError {
    /// 将错误转换为用户友好的消息
    pub fn to_user_message(&self) -> &'static str {
        use crate::systems::text_constants::SaveLoadText;

        match self {
            SaveSystemError::FileNotFound(_) => SaveLoadText::FILE_NOT_FOUND_ERROR,
            SaveSystemError::PermissionDenied(_) => SaveLoadText::PERMISSION_DENIED_ERROR,
            SaveSystemError::DiskSpaceInsufficient => "Insufficient disk space",
            SaveSystemError::FileCorrupted(_) => SaveLoadText::CORRUPTED_FILE_ERROR,
            SaveSystemError::DirectoryCreationFailed(_) => "Failed to create directory",
            SaveSystemError::FileWriteFailed(_) => "Failed to write file",
            SaveSystemError::DirectoryReadFailed(_) => "Failed to read directory",

            SaveSystemError::SerializationFailed(_) => SaveLoadText::SAVE_ERROR,
            SaveSystemError::DeserializationFailed(_) => SaveLoadText::LOAD_ERROR,

            SaveSystemError::CompressionFailed(_) => "Failed to compress data",
            SaveSystemError::DecompressionFailed(_) => "Failed to decompress data",

            SaveSystemError::InvalidFileName(_) => SaveLoadText::INVALID_NAME_ERROR,
            SaveSystemError::ChecksumMismatch => "Save file integrity check failed",
            SaveSystemError::VersionMismatch(_) => "Incompatible save file version",
            SaveSystemError::NameAlreadyExists(_) => SaveLoadText::NAME_EXISTS_ERROR,

            SaveSystemError::TextInputFailed(_) => "Text input error",
            SaveSystemError::KeyboardInputError(_) => "Keyboard input error",

            SaveSystemError::ResourceNotFound(_) => "Required resource not found",
            SaveSystemError::StateTransitionError(_) => "Game state transition error",
            SaveSystemError::AudioSystemError(_) => "Audio system error",
        }
    }

    /// 获取详细的错误信息（用于调试）
    pub fn get_details(&self) -> String {
        match self {
            SaveSystemError::FileNotFound(path) => format!("File not found: {}", path),
            SaveSystemError::PermissionDenied(path) => format!("Permission denied: {}", path),
            SaveSystemError::FileCorrupted(path) => format!("Corrupted file: {}", path),
            SaveSystemError::DirectoryCreationFailed(path) => {
                format!("Failed to create directory: {}", path)
            }
            SaveSystemError::FileWriteFailed(msg) => format!("Failed to write file: {}", msg),
            SaveSystemError::DirectoryReadFailed(msg) => {
                format!("Failed to read directory: {}", msg)
            }

            SaveSystemError::SerializationFailed(msg) => format!("Serialization failed: {}", msg),
            SaveSystemError::DeserializationFailed(msg) => {
                format!("Deserialization failed: {}", msg)
            }

            SaveSystemError::CompressionFailed(msg) => format!("Compression failed: {}", msg),
            SaveSystemError::DecompressionFailed(msg) => format!("Decompression failed: {}", msg),

            SaveSystemError::InvalidFileName(name) => format!("Invalid file name: {}", name),
            SaveSystemError::VersionMismatch(version) => format!("Version mismatch: {}", version),
            SaveSystemError::NameAlreadyExists(name) => format!("Name already exists: {}", name),

            SaveSystemError::TextInputFailed(msg) => format!("Text input failed: {}", msg),
            SaveSystemError::KeyboardInputError(msg) => format!("Keyboard input error: {}", msg),

            SaveSystemError::ResourceNotFound(resource) => {
                format!("Resource not found: {}", resource)
            }
            SaveSystemError::StateTransitionError(msg) => {
                format!("State transition error: {}", msg)
            }
            SaveSystemError::AudioSystemError(msg) => format!("Audio system error: {}", msg),

            SaveSystemError::DiskSpaceInsufficient => "Insufficient disk space".to_string(),
            SaveSystemError::ChecksumMismatch => "Checksum mismatch".to_string(),
        }
    }

    /// 判断错误是否可以重试
    pub fn is_retryable(&self) -> bool {
        match self {
            SaveSystemError::DiskSpaceInsufficient => false,
            SaveSystemError::PermissionDenied(_) => false,
            SaveSystemError::FileCorrupted(_) => false,
            SaveSystemError::ChecksumMismatch => false,
            SaveSystemError::VersionMismatch(_) => false,
            SaveSystemError::InvalidFileName(_) => false,
            SaveSystemError::NameAlreadyExists(_) => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for SaveSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_details())
    }
}

impl std::error::Error for SaveSystemError {}

/// 错误恢复管理器
#[derive(Resource)]
pub struct ErrorRecoveryManager {
    pub retry_attempts: HashMap<String, u32>,
    pub max_retries: u32,
    pub backup_directory: PathBuf,
    pub error_history: Vec<ErrorRecord>,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self {
            retry_attempts: HashMap::new(),
            max_retries: 3,
            backup_directory: PathBuf::from("saves/backup"),
            error_history: Vec::new(),
        }
    }

    /// 处理保存错误
    pub fn handle_save_error(&mut self, error: SaveSystemError, operation: &str) -> RecoveryAction {
        self.log_error(&error, operation);

        if error.is_retryable() {
            let attempts = self
                .retry_attempts
                .entry(operation.to_string())
                .or_insert(0);
            *attempts += 1;

            if *attempts <= self.max_retries {
                println!(
                    "🔄 Retrying save operation (attempt {}/{})",
                    attempts, self.max_retries
                );
                return RecoveryAction::Retry;
            } else {
                println!("❌ Max retries exceeded for save operation");
                self.retry_attempts.remove(operation);
                return RecoveryAction::ShowError(error.to_user_message().to_string());
            }
        }

        match error {
            SaveSystemError::DiskSpaceInsufficient => {
                RecoveryAction::ShowError("Please free up disk space and try again".to_string())
            }
            SaveSystemError::PermissionDenied(_) => {
                RecoveryAction::ShowError("Please check file permissions and try again".to_string())
            }
            SaveSystemError::NameAlreadyExists(_) => {
                RecoveryAction::ShowError("Please choose a different name".to_string())
            }
            _ => RecoveryAction::ShowError(error.to_user_message().to_string()),
        }
    }

    /// 处理加载错误
    pub fn handle_load_error(&mut self, error: SaveSystemError, operation: &str) -> RecoveryAction {
        self.log_error(&error, operation);

        match error {
            SaveSystemError::FileNotFound(_) => RecoveryAction::ShowError(
                "Save file not found. It may have been deleted.".to_string(),
            ),
            SaveSystemError::FileCorrupted(_) => {
                if self.backup_exists(operation) {
                    RecoveryAction::UseBackup
                } else {
                    RecoveryAction::ShowError(
                        "Save file is corrupted and no backup is available".to_string(),
                    )
                }
            }
            SaveSystemError::ChecksumMismatch => RecoveryAction::ShowError(
                "Save file integrity check failed. File may be corrupted.".to_string(),
            ),
            SaveSystemError::VersionMismatch(_) => {
                RecoveryAction::ShowError("Save file is from an incompatible version".to_string())
            }
            _ => {
                if error.is_retryable() {
                    let attempts = self
                        .retry_attempts
                        .entry(operation.to_string())
                        .or_insert(0);
                    *attempts += 1;

                    if *attempts <= self.max_retries {
                        RecoveryAction::Retry
                    } else {
                        self.retry_attempts.remove(operation);
                        RecoveryAction::ShowError(error.to_user_message().to_string())
                    }
                } else {
                    RecoveryAction::ShowError(error.to_user_message().to_string())
                }
            }
        }
    }

    /// 创建备份
    pub fn create_backup(&self, save_name: &str, data: &str) -> Result<(), SaveSystemError> {
        use std::fs;

        // 确保备份目录存在
        if !self.backup_directory.exists() {
            fs::create_dir_all(&self.backup_directory)
                .map_err(|e| SaveSystemError::DirectoryCreationFailed(e.to_string()))?;
        }

        // 创建备份文件
        let backup_file = self
            .backup_directory
            .join(format!("{}_backup.json", save_name));
        fs::write(&backup_file, data)
            .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

        println!("💾 Backup created: {}", backup_file.display());
        Ok(())
    }

    /// 从备份恢复
    pub fn restore_from_backup(&self, save_name: &str) -> Result<String, SaveSystemError> {
        use std::fs;

        let backup_file = self
            .backup_directory
            .join(format!("{}_backup.json", save_name));

        if !backup_file.exists() {
            return Err(SaveSystemError::FileNotFound(
                backup_file.to_string_lossy().to_string(),
            ));
        }

        let data = fs::read_to_string(&backup_file)
            .map_err(|e| SaveSystemError::DeserializationFailed(e.to_string()))?;

        println!("📂 Restored from backup: {}", backup_file.display());
        Ok(data)
    }

    /// 检查备份是否存在
    fn backup_exists(&self, save_name: &str) -> bool {
        let backup_file = self
            .backup_directory
            .join(format!("{}_backup.json", save_name));
        backup_file.exists()
    }

    /// 记录错误
    fn log_error(&mut self, error: &SaveSystemError, operation: &str) {
        let record = ErrorRecord {
            error: error.clone(),
            operation: operation.to_string(),
            timestamp: chrono::Utc::now(),
        };

        self.error_history.push(record);

        // 保持错误历史记录在合理范围内
        if self.error_history.len() > 100 {
            self.error_history.remove(0);
        }

        println!("❌ Error logged: {} - {}", operation, error.get_details());
    }

    /// 清除重试计数
    pub fn clear_retry_count(&mut self, operation: &str) {
        self.retry_attempts.remove(operation);
    }

    /// 获取错误统计
    pub fn get_error_stats(&self) -> ErrorStats {
        let mut stats = ErrorStats::default();

        for record in &self.error_history {
            stats.total_errors += 1;
            match record.error {
                SaveSystemError::FileNotFound(_) => stats.file_errors += 1,
                SaveSystemError::PermissionDenied(_) => stats.permission_errors += 1,
                SaveSystemError::FileCorrupted(_) => stats.corruption_errors += 1,
                SaveSystemError::SerializationFailed(_)
                | SaveSystemError::DeserializationFailed(_) => stats.serialization_errors += 1,
                _ => stats.other_errors += 1,
            }
        }

        stats
    }
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 恢复操作类型
#[derive(Debug)]
pub enum RecoveryAction {
    Retry,
    UseBackup,
    ShowError(String),
    ReturnToMenu,
}

/// 错误记录
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    pub error: SaveSystemError,
    pub operation: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 错误统计
#[derive(Debug, Default)]
pub struct ErrorStats {
    pub total_errors: u32,
    pub file_errors: u32,
    pub permission_errors: u32,
    pub corruption_errors: u32,
    pub serialization_errors: u32,
    pub other_errors: u32,
}

/// 错误处理系统
pub fn handle_system_errors(mut error_recovery: ResMut<ErrorRecoveryManager>) {
    // 这个系统可以用来处理全局错误状态
    // 例如，定期清理旧的错误记录或重试计数

    // 清理超过24小时的错误记录
    let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(24);
    error_recovery
        .error_history
        .retain(|record| record.timestamp > cutoff_time);
}

/// 将标准错误转换为SaveSystemError
pub fn convert_io_error(error: std::io::Error, context: &str) -> SaveSystemError {
    match error.kind() {
        std::io::ErrorKind::NotFound => SaveSystemError::FileNotFound(context.to_string()),
        std::io::ErrorKind::PermissionDenied => {
            SaveSystemError::PermissionDenied(context.to_string())
        }
        std::io::ErrorKind::InvalidData => SaveSystemError::FileCorrupted(context.to_string()),
        _ => SaveSystemError::SerializationFailed(format!("{}: {}", context, error)),
    }
}
