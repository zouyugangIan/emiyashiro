//! 碰撞检测系统
//!
//! 包含游戏中所有的碰撞检测逻辑，提供精确和高效的碰撞处理。

use crate::{components::*, resources::*};
use bevy::prelude::*;

type GroundCollisionQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static CollisionBox), (With<Ground>, Without<Player>)>;

/// 碰撞检测结果
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// 是否发生碰撞
    pub collided: bool,
    /// 碰撞点
    pub contact_point: Vec2,
    /// 碰撞法向量
    pub normal: Vec2,
    /// 穿透深度
    pub penetration: f32,
}

impl Default for CollisionResult {
    fn default() -> Self {
        Self {
            collided: false,
            contact_point: Vec2::ZERO,
            normal: Vec2::ZERO,
            penetration: 0.0,
        }
    }
}

/// 碰撞盒组件
///
/// 定义实体的碰撞边界。
#[derive(Component, Debug, Clone)]
pub struct CollisionBox {
    /// 碰撞盒的大小
    pub size: Vec2,
    /// 相对于实体中心的偏移
    pub offset: Vec2,
    /// 是否为触发器（不阻挡移动，只检测碰撞）
    pub is_trigger: bool,
}

impl CollisionBox {
    /// 创建新的碰撞盒
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            offset: Vec2::ZERO,
            is_trigger: false,
        }
    }

    /// 创建触发器碰撞盒
    pub fn trigger(size: Vec2) -> Self {
        Self {
            size,
            offset: Vec2::ZERO,
            is_trigger: true,
        }
    }

    /// 设置偏移
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// 获取世界空间中的碰撞盒
    pub fn world_bounds(&self, position: Vec3) -> Rect {
        let center = Vec2::new(position.x, position.y) + self.offset;
        let _half_size = self.size * 0.5;

        Rect::from_center_size(center, self.size)
    }
}

/// 碰撞检测系统
///
/// 检测玩家与地面和其他物体的碰撞。
pub fn collision_detection_system(
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PlayerState,
            &CollisionBox,
        ),
        With<Player>,
    >,
    ground_query: GroundCollisionQuery,
) {
    if let Ok((mut player_transform, mut player_velocity, mut player_state, player_collision)) =
        player_query.single_mut()
    {
        let mut on_ground = false;

        // 获取玩家的碰撞盒
        let player_bounds = player_collision.world_bounds(player_transform.translation);

        // 检测与地面的碰撞
        for (ground_transform, ground_collision) in ground_query.iter() {
            let ground_bounds = ground_collision.world_bounds(ground_transform.translation);

            let collision = check_aabb_collision(&player_bounds, &ground_bounds);

            if collision.collided && !ground_collision.is_trigger {
                // 解决碰撞
                resolve_collision(&mut player_transform, &mut player_velocity, &collision);

                // 检查是否在地面上
                if collision.normal.y > 0.5 {
                    on_ground = true;
                }
            }
        }

        // 更新玩家状态
        player_state.is_grounded = on_ground;
    }
}

/// AABB（轴对齐包围盒）碰撞检测
///
/// 检测两个矩形是否碰撞。
fn check_aabb_collision(rect1: &Rect, rect2: &Rect) -> CollisionResult {
    let mut result = CollisionResult::default();

    // 检查是否重叠
    if rect1.max.x < rect2.min.x
        || rect1.min.x > rect2.max.x
        || rect1.max.y < rect2.min.y
        || rect1.min.y > rect2.max.y
    {
        return result; // 没有碰撞
    }

    result.collided = true;

    // 计算重叠区域
    let overlap_x = (rect1.max.x - rect2.min.x).min(rect2.max.x - rect1.min.x);
    let overlap_y = (rect1.max.y - rect2.min.y).min(rect2.max.y - rect1.min.y);

    // 确定碰撞方向（最小穿透方向）
    if overlap_x < overlap_y {
        // 水平碰撞
        result.penetration = overlap_x;
        result.normal = if rect1.center().x < rect2.center().x {
            Vec2::new(-1.0, 0.0) // 从右侧碰撞
        } else {
            Vec2::new(1.0, 0.0) // 从左侧碰撞
        };
        result.contact_point = Vec2::new(
            if rect1.center().x < rect2.center().x {
                rect2.min.x
            } else {
                rect2.max.x
            },
            rect1.center().y,
        );
    } else {
        // 垂直碰撞
        result.penetration = overlap_y;
        result.normal = if rect1.center().y < rect2.center().y {
            Vec2::new(0.0, -1.0) // 从上方碰撞
        } else {
            Vec2::new(0.0, 1.0) // 从下方碰撞
        };
        result.contact_point = Vec2::new(
            rect1.center().x,
            if rect1.center().y < rect2.center().y {
                rect2.min.y
            } else {
                rect2.max.y
            },
        );
    }

    result
}

/// 解决碰撞
///
/// 根据碰撞结果调整实体位置和速度。
fn resolve_collision(
    transform: &mut Transform,
    velocity: &mut Velocity,
    collision: &CollisionResult,
) {
    // 移动实体出碰撞区域
    let separation = collision.normal * collision.penetration;
    transform.translation.x += separation.x;
    transform.translation.y += separation.y;

    // 调整速度（移除朝向碰撞面的速度分量）
    let velocity_dot_normal = velocity.x * collision.normal.x + velocity.y * collision.normal.y;

    if velocity_dot_normal < 0.0 {
        velocity.x -= velocity_dot_normal * collision.normal.x;
        velocity.y -= velocity_dot_normal * collision.normal.y;
    }
}

/// 高级碰撞检测系统
///
/// 提供更精确的碰撞检测，包括斜坡和复杂形状。
pub fn advanced_collision_system(
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut velocity, mut player_state)) = player_query.single_mut() {
        let delta_time = time.delta_secs();

        // 预测下一帧的位置
        let predicted_position = Vec3::new(
            transform.translation.x + velocity.x * delta_time,
            transform.translation.y + velocity.y * delta_time,
            transform.translation.z,
        );

        // 检查预测位置是否会导致碰撞
        if check_predicted_collision(&predicted_position) {
            // 使用更小的时间步长进行精确碰撞检测
            let substeps = 4;
            let substep_time = delta_time / substeps as f32;

            for _ in 0..substeps {
                let next_pos = Vec3::new(
                    transform.translation.x + velocity.x * substep_time,
                    transform.translation.y + velocity.y * substep_time,
                    transform.translation.z,
                );

                if !check_predicted_collision(&next_pos) {
                    transform.translation = next_pos;
                } else {
                    // 发生碰撞，停止移动
                    break;
                }
            }
        }

        // 地面检测（使用射线投射）
        let ground_check_result = raycast_ground_check(&transform.translation);
        player_state.is_grounded = ground_check_result.hit;

        if ground_check_result.hit {
            let ground_distance = ground_check_result.distance;
            let _player_bottom = transform.translation.y - GameConfig::PLAYER_SIZE.y * 0.5;

            // 如果玩家在地面上或稍微穿透地面
            if ground_distance < 5.0 {
                transform.translation.y =
                    ground_check_result.point.y + GameConfig::PLAYER_SIZE.y * 0.5;
                if velocity.y < 0.0 {
                    velocity.y = 0.0;
                }
            }
        }
    }
}

/// 射线投射结果
#[derive(Debug, Clone)]
pub struct RaycastResult {
    /// 是否命中
    pub hit: bool,
    /// 命中点
    pub point: Vec3,
    /// 命中距离
    pub distance: f32,
    /// 命中法向量
    pub normal: Vec3,
}

impl Default for RaycastResult {
    fn default() -> Self {
        Self {
            hit: false,
            point: Vec3::ZERO,
            distance: f32::INFINITY,
            normal: Vec3::ZERO,
        }
    }
}

/// 地面射线检测
///
/// 从玩家位置向下发射射线，检测地面。
fn raycast_ground_check(position: &Vec3) -> RaycastResult {
    let mut result = RaycastResult::default();

    // 简化的地面检测（实际游戏中可能需要更复杂的实现）
    let ground_y = GameConfig::GROUND_LEVEL;
    let ray_start_y = position.y - GameConfig::PLAYER_SIZE.y * 0.5;

    if ray_start_y > ground_y {
        result.hit = true;
        result.point = Vec3::new(position.x, ground_y, position.z);
        result.distance = ray_start_y - ground_y;
        result.normal = Vec3::Y;
    }

    result
}

/// 预测碰撞检查
///
/// 检查给定位置是否会发生碰撞。
fn check_predicted_collision(position: &Vec3) -> bool {
    // 简化的碰撞检查
    // 检查是否会穿透地面
    let player_bottom = position.y - GameConfig::PLAYER_SIZE.y * 0.5;

    if player_bottom < GameConfig::GROUND_LEVEL {
        return true;
    }

    // 检查边界
    if position.x.abs() > 5000.0 {
        return true;
    }

    false
}

/// 碰撞调试系统
///
/// 在开发模式下显示碰撞盒和碰撞信息。
pub fn collision_debug_system(
    player_query: Query<(&Transform, &CollisionBox), With<Player>>,
    ground_query: Query<(&Transform, &CollisionBox), With<Ground>>,
    mut gizmos: Gizmos,
) {
    // 绘制玩家碰撞盒
    if let Ok((transform, collision_box)) = player_query.single() {
        let bounds = collision_box.world_bounds(transform.translation);
        gizmos.rect_2d(
            bounds.center(),
            bounds.size(),
            Color::srgb(0.0, 1.0, 0.0), // 绿色
        );
    }

    // 绘制地面碰撞盒
    for (transform, collision_box) in ground_query.iter() {
        let bounds = collision_box.world_bounds(transform.translation);
        gizmos.rect_2d(
            bounds.center(),
            bounds.size(),
            Color::srgb(1.0, 0.0, 0.0), // 红色
        );
    }
}
