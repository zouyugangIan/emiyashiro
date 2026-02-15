use bevy::prelude::*;

/// 统一存档系统的系统集合
/// 定义系统执行顺序以确保正确的依赖关系
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum UnifiedSaveSystemSet {
    /// 输入处理 - 最先执行
    Input,
    /// 文本处理 - 处理键盘输入后的文本
    TextProcessing,
    /// 状态捕获 - 捕获游戏状态
    StateCapture,
    /// 文件操作 - 异步文件读写
    FileOperations,
    /// UI更新 - 更新用户界面
    UIUpdate,
    /// 音频管理 - 管理音频状态
    AudioManagement,
    /// 错误处理 - 处理系统错误
    ErrorHandling,
}

/// 配置系统调度顺序
pub fn configure_save_system_scheduling(app: &mut App) {
    app.configure_sets(
        Update,
        (
            UnifiedSaveSystemSet::Input,
            UnifiedSaveSystemSet::TextProcessing,
            UnifiedSaveSystemSet::StateCapture,
            UnifiedSaveSystemSet::FileOperations,
            UnifiedSaveSystemSet::UIUpdate,
            UnifiedSaveSystemSet::AudioManagement,
            UnifiedSaveSystemSet::ErrorHandling,
        )
            .chain(),
    );
}

/// 游戏状态系统集合
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameStateSystemSet {
    /// 菜单系统
    Menu,
    /// 游戏进行中系统
    Playing,
    /// 暂停系统
    Paused,
    /// 保存对话框系统
    SaveDialog,
    /// 加载表格系统
    LoadTable,
    /// 重命名对话框系统
    RenameDialog,
}

/// 性能优化系统集合
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PerformanceSystemSet {
    /// 高频率系统（每帧执行）
    HighFrequency,
    /// 中频率系统（每几帧执行一次）
    MediumFrequency,
    /// 低频率系统（定期执行）
    LowFrequency,
}

/// 配置性能优化调度
pub fn configure_performance_scheduling(app: &mut App) {
    app.configure_sets(
        Update,
        (
            PerformanceSystemSet::HighFrequency,
            PerformanceSystemSet::MediumFrequency,
            PerformanceSystemSet::LowFrequency,
        )
            .chain(),
    );
}
