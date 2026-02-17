//! 系统接口定义
//!
//! 定义游戏系统的标准接口和系统集合，用于统一管理系统的执行顺序和依赖关系。

use bevy::prelude::*;

/// 游戏系统集合
///
/// 定义不同类型系统的执行顺序，确保系统间的正确依赖关系。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
    /// 输入处理系统集合
    ///
    /// 最先执行，处理用户输入和输入状态更新
    Input,

    /// 游戏逻辑系统集合
    ///
    /// 处理核心游戏逻辑，如玩家移动、物理计算等
    GameLogic,

    /// 动画系统集合
    ///
    /// 处理角色动画和视觉效果更新
    Animation,

    /// 摄像机系统集合
    ///
    /// 处理摄像机跟随和视角控制
    Camera,

    /// UI系统集合
    ///
    /// 处理用户界面更新和交互
    UI,

    /// 音频系统集合
    ///
    /// 处理音效播放和音频状态管理
    Audio,

    /// 数据持久化系统集合
    ///
    /// 处理存档和数据库操作
    Persistence,
}

/// 系统接口特征
///
/// 定义系统的标准接口，用于统一管理和调度。
pub trait GameSystem {
    /// 系统名称
    fn name(&self) -> &'static str;

    /// 系统描述
    fn description(&self) -> &'static str;

    /// 系统所属的集合
    fn system_set(&self) -> GameSystemSet;

    /// 系统是否启用
    fn is_enabled(&self) -> bool {
        true
    }
}

/// 状态相关的系统接口
///
/// 用于定义在特定游戏状态下运行的系统。
pub trait StateSystem<T: States>: GameSystem {
    /// 系统运行的状态
    fn target_state(&self) -> T;
}

/// 系统配置辅助函数
pub struct SystemConfig;

impl SystemConfig {
    /// 配置输入系统
    ///
    /// 设置输入处理相关系统的执行顺序和依赖关系。
    pub fn configure_input_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::Input.before(GameSystemSet::GameLogic),
        );
    }

    /// 配置游戏逻辑系统
    ///
    /// 设置核心游戏逻辑系统的执行顺序。
    pub fn configure_game_logic_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::GameLogic
                .after(GameSystemSet::Input)
                .before(GameSystemSet::Animation),
        );
    }

    /// 配置动画系统
    ///
    /// 设置动画相关系统的执行顺序。
    pub fn configure_animation_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::Animation
                .after(GameSystemSet::GameLogic)
                .before(GameSystemSet::Camera),
        );
    }

    /// 配置摄像机系统
    ///
    /// 设置摄像机控制系统的执行顺序。
    pub fn configure_camera_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::Camera
                .after(GameSystemSet::Animation)
                .before(GameSystemSet::UI),
        );
    }

    /// 配置UI系统
    ///
    /// 设置用户界面系统的执行顺序。
    pub fn configure_ui_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::UI
                .after(GameSystemSet::Camera)
                .before(GameSystemSet::Audio),
        );
    }

    /// 配置音频系统
    ///
    /// 设置音频处理系统的执行顺序。
    pub fn configure_audio_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::Audio
                .after(GameSystemSet::UI)
                .before(GameSystemSet::Persistence),
        );
    }

    /// 配置数据持久化系统
    ///
    /// 设置存档和数据库系统的执行顺序。
    pub fn configure_persistence_systems(app: &mut App) {
        app.configure_sets(
            Update,
            GameSystemSet::Persistence.after(GameSystemSet::Audio),
        );
    }

    /// 配置所有系统集合
    ///
    /// 一次性配置所有系统的执行顺序和依赖关系。
    pub fn configure_all_systems(app: &mut App) {
        Self::configure_input_systems(app);
        Self::configure_game_logic_systems(app);
        Self::configure_animation_systems(app);
        Self::configure_camera_systems(app);
        Self::configure_ui_systems(app);
        Self::configure_audio_systems(app);
        Self::configure_persistence_systems(app);
    }
}

/// 系统性能监控
///
/// 用于监控系统的执行性能和资源使用情况。
#[derive(Resource, Default)]
pub struct SystemPerformanceMonitor {
    /// 系统执行时间记录
    pub execution_times: std::collections::HashMap<String, f32>,
    /// 系统调用次数
    pub call_counts: std::collections::HashMap<String, u32>,
    /// 性能监控是否启用
    pub enabled: bool,
}

impl SystemPerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            execution_times: std::collections::HashMap::new(),
            call_counts: std::collections::HashMap::new(),
            enabled: false,
        }
    }

    /// 启用性能监控
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用性能监控
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 记录系统执行时间
    pub fn record_execution_time(&mut self, system_name: &str, duration: f32) {
        if self.enabled {
            *self
                .execution_times
                .entry(system_name.to_string())
                .or_insert(0.0) += duration;
            *self.call_counts.entry(system_name.to_string()).or_insert(0) += 1;
        }
    }

    /// 获取系统平均执行时间
    pub fn get_average_execution_time(&self, system_name: &str) -> Option<f32> {
        if let (Some(&total_time), Some(&call_count)) = (
            self.execution_times.get(system_name),
            self.call_counts.get(system_name),
        ) {
            if call_count > 0 {
                Some(total_time / call_count as f32)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 重置性能统计
    pub fn reset(&mut self) {
        self.execution_times.clear();
        self.call_counts.clear();
    }

    /// 打印性能报告
    pub fn print_performance_report(&self) {
        if !self.enabled {
            return;
        }

        crate::debug_log!("=== 系统性能报告 ===");
        for (system_name, &total_time) in &self.execution_times {
            if let Some(&call_count) = self.call_counts.get(system_name) {
                let avg_time = total_time / call_count as f32;
                crate::debug_log!(
                    "{}: 总时间 {:.3}ms, 调用次数 {}, 平均时间 {:.3}ms",
                    system_name,
                    total_time * 1000.0,
                    call_count,
                    avg_time * 1000.0
                );
            }
        }
        crate::debug_log!("==================");
    }
}
