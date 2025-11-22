use crate::states::GameState;
use crate::systems::{
    async_file_ops::*, error_handling::*, pause_save::*, system_sets::*, text_input::*, ui::*,
};
use bevy::prelude::*;

/// 最终集成插件
/// 将所有统一存档系统组件整合到一个插件中
pub struct UnifiedSaveSystemPlugin;

impl Plugin for UnifiedSaveSystemPlugin {
    fn build(&self, app: &mut App) {
        // 配置系统调度
        configure_save_system_scheduling(app);
        configure_performance_scheduling(app);

        // 初始化资源
        app.init_resource::<AsyncFileManager>()
            .init_resource::<OperationProgress>()
            .init_resource::<ErrorRecoveryManager>()
            .init_resource::<TextInputState>()
            .init_resource::<KeyboardInputHandler>()
            .init_resource::<crate::resources::SaveFileManager>()
            .init_resource::<crate::resources::PauseManager>()
            .init_resource::<crate::resources::AudioStateManager>();

        // 添加输入处理系统
        app.add_systems(
            Update,
            (
                handle_keyboard_input,
                crate::systems::input::update_game_input,
            )
                .in_set(UnifiedSaveSystemSet::Input),
        );

        // 添加文本处理系统
        app.add_systems(
            Update,
            (
                update_text_cursor,
                handle_save_name_input,
                handle_rename_input,
            )
                .in_set(UnifiedSaveSystemSet::TextProcessing),
        );

        // 添加状态捕获系统
        app.add_systems(
            Update,
            (handle_pause_input, restore_paused_state)
                .in_set(UnifiedSaveSystemSet::StateCapture)
                .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
        );

        // 添加文件操作系统
        app.add_systems(
            Update,
            (update_operation_progress,).in_set(UnifiedSaveSystemSet::FileOperations),
        );

        // 添加UI更新系统
        app.add_systems(
            Update,
            (
                display_progress_indicator,
                handle_pause_menu_interactions,
                handle_save_dialog_interactions,
                handle_load_table_interactions,
                handle_rename_dialog_interactions,
            )
                .in_set(UnifiedSaveSystemSet::UIUpdate),
        );

        // 添加音频管理系统
        app.add_systems(
            Update,
            crate::systems::audio::maintain_audio_during_pause
                .in_set(UnifiedSaveSystemSet::AudioManagement)
                .run_if(in_state(GameState::Paused)),
        );

        // 添加错误处理系统
        app.add_systems(
            Update,
            handle_system_errors.in_set(UnifiedSaveSystemSet::ErrorHandling),
        );

        // 状态转换系统
        app.add_systems(
            OnEnter(GameState::Paused),
            (setup_pause_menu, scan_save_files),
        );

        app.add_systems(OnExit(GameState::Paused), cleanup_pause_menu);

        app.add_systems(OnEnter(GameState::SaveDialog), setup_save_dialog);
        app.add_systems(OnExit(GameState::SaveDialog), cleanup_save_dialog);

        app.add_systems(
            OnEnter(GameState::LoadTable),
            (setup_load_table, scan_save_files),
        );
        app.add_systems(OnExit(GameState::LoadTable), cleanup_load_table);

        app.add_systems(OnEnter(GameState::RenameDialog), setup_rename_dialog);
        app.add_systems(OnExit(GameState::RenameDialog), cleanup_rename_dialog);
    }
}

/// 性能监控资源
#[derive(Resource, Default)]
pub struct PerformanceMonitor {
    pub frame_time: f32,
    pub save_operation_time: f32,
    pub load_operation_time: f32,
    pub ui_update_time: f32,
    pub memory_usage: u64,
}

impl PerformanceMonitor {
    pub fn update_frame_time(&mut self, delta_time: f32) {
        self.frame_time = delta_time;
    }

    pub fn get_fps(&self) -> f32 {
        if self.frame_time > 0.0 {
            1.0 / self.frame_time
        } else {
            0.0
        }
    }

    pub fn is_performance_good(&self) -> bool {
        self.get_fps() >= 55.0 && self.save_operation_time < 1.0
    }
}

/// 性能监控系统
pub fn monitor_performance(
    mut monitor: ResMut<PerformanceMonitor>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    monitor.update_frame_time(time.delta_secs());

    // 初始化计时器（每5秒检查一次）
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }
    timer.tick(time.delta());

    // 只在计时器触发时记录警告
    if timer.just_finished() && !monitor.is_performance_good() {
        warn!(
            "Performance warning: FPS: {:.1}, Save time: {:.2}s",
            monitor.get_fps(),
            monitor.save_operation_time
        );
    }
}

/// 内存管理系统
pub fn manage_memory(mut commands: Commands, query: Query<Entity, With<Node>>) {
    // 清理孤立的UI节点
    let mut orphaned_count = 0;
    for entity in query.iter() {
        // 检查是否为孤立节点（这里简化处理）
        if orphaned_count > 100 {
            // 假设超过100个就需要清理
            commands.entity(entity).despawn();
            orphaned_count += 1;
        }
    }

    if orphaned_count > 0 {
        info!("Cleaned up {} orphaned UI nodes", orphaned_count);
    }
}

/// 自动保存配置
#[derive(Resource)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_seconds: f32,
    pub max_auto_saves: usize,
    pub last_auto_save: f32,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300.0, // 5分钟
            max_auto_saves: 5,
            last_auto_save: 0.0,
        }
    }
}

/// 自动保存系统
pub fn auto_save_system(
    mut config: ResMut<AutoSaveConfig>,
    time: Res<Time>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    _pause_manager: Res<crate::resources::PauseManager>,
) {
    if !config.enabled || *current_state.get() != GameState::Playing {
        return;
    }

    let current_time = time.elapsed_secs();

    if current_time - config.last_auto_save >= config.interval_seconds {
        // 触发自动保存
        info!("Triggering auto-save...");

        // 暂时切换到暂停状态进行保存
        next_state.set(GameState::Paused);

        // 这里应该触发实际的保存操作
        // 实际实现中会调用异步保存函数

        config.last_auto_save = current_time;
    }
}

/// 系统健康检查
pub fn system_health_check(
    error_recovery: Res<ErrorRecoveryManager>,
    performance: Res<PerformanceMonitor>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    // 初始化计时器（每30秒检查一次）
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(30.0, TimerMode::Repeating);
    }
    timer.tick(time.delta());

    // 只在计时器触发时输出健康状态
    if !timer.just_finished() {
        return;
    }

    let stats = error_recovery.get_error_stats();

    // 检查错误率
    if stats.total_errors > 10 {
        warn!("High error count detected: {}", stats.total_errors);
    }

    // 检查性能
    if !performance.is_performance_good() {
        warn!("Performance issues detected");
    }

    // 定期输出健康状态（降低频率）
    if stats.total_errors == 0 && performance.is_performance_good() {
        info!("System health: Good (FPS: {:.1})", performance.get_fps());
    }
}

/// 配置最终集成
pub fn configure_final_integration(app: &mut App) {
    app.add_plugins(UnifiedSaveSystemPlugin)
        .init_resource::<PerformanceMonitor>()
        .init_resource::<AutoSaveConfig>()
        .add_systems(
            Update,
            (
                monitor_performance,
                manage_memory,
                auto_save_system,
                system_health_check,
            )
                .in_set(PerformanceSystemSet::LowFrequency),
        );
}
