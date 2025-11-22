# Design Document: Menu UI Responsive Refactor

## Overview

本设计文档描述了将游戏主菜单系统从基于Sprite的固定尺寸实现重构为基于Bevy UI节点的响应式实现。重构的核心目标是实现完全响应式的布局，支持任意窗口大小，同时保持现有的封面渐变动画效果和所有交互功能。

当前实现使用 `Sprite` 组件配合固定的 `custom_size: Some(Vec2::new(1024.0, 768.0))` 来显示封面图片，这导致在不同窗口大小下无法正确适配。重构后将使用 `ImageNode` 和百分比布局来实现真正的响应式设计。

## Architecture

### 高层架构

```
MenuSystem (UI Root)
├── Background Layer (ImageNode-based)
│   ├── CoverImage1 (100% width/height, z-index: 0)
│   └── CoverImage2 (100% width/height, z-index: 1)
└── Interactive Layer (UI Nodes)
    ├── Title Text
    ├── Button Container
    │   ├── Start Button
    │   └── Load Button
    └── Character Select Container
        ├── Character 1 Button
        └── Character 2 Button
```

### 关键设计决策

1. **分层架构**: 将背景层和交互层分离，背景层使用绝对定位的UI节点填充整个窗口，交互层使用flexbox布局居中显示
2. **Z-ordering**: 使用UI节点的父子层级关系管理Z轴顺序，而不是依赖Transform的z坐标
3. **动画系统**: 修改动画系统从操作 `Sprite::color` 改为操作 `BackgroundColor` 组件
4. **组件复用**: 保持现有的 `CoverImage1`, `CoverImage2`, `CoverFadeState` 组件定义不变，仅修改查询类型

## Components and Interfaces

### 修改的组件

#### CoverImage1 和 CoverImage2
这两个标记组件保持不变，但将附加到UI节点而不是Sprite实体：

```rust
#[derive(Component, Debug)]
pub struct CoverImage1;

#[derive(Component, Debug)]
pub struct CoverImage2;
```

#### CoverFadeState
保持不变，继续用于存储渐变动画状态：

```rust
#[derive(Component, Debug, Clone)]
pub struct CoverFadeState {
    pub alpha: f32,
    pub fade_direction: f32,
}
```

### 新增的Bevy UI组件使用

重构后将使用以下Bevy内置UI组件：

- `Node`: 定义UI元素的布局属性（宽度、高度、对齐方式等）
- `ImageNode`: 在UI节点中显示图像
- `BackgroundColor`: 控制UI节点的背景颜色和透明度
- `ZIndex`: 显式控制UI元素的层级顺序

## Data Models

### UI节点结构

#### 背景层节点
```rust
// 第一张封面图片
commands.spawn((
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    },
    ImageNode::new(cover_texture),
    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
    ZIndex(0),
    MenuUI,
    CoverImage1,
    CoverFadeState::default(),
));

// 第二张封面图片
commands.spawn((
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    },
    ImageNode::new(cover2_texture),
    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)), // 初始透明
    ZIndex(1),
    MenuUI,
    CoverImage2,
    CoverFadeState::new(0.0, -1.0),
));
```

#### 交互层根节点
```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Absolute,
        ..default()
    },
    ZIndex(2), // 确保在封面图片之上
    MenuUI,
))
```

### 动画数据流

```
Time Resource
    ↓
cover_fade_animation system
    ↓
Query<(&mut BackgroundColor, &mut CoverFadeState), Or<(With<CoverImage1>, With<CoverImage2>)>>
    ↓
计算新的alpha值
    ↓
更新 BackgroundColor.0.set_alpha(alpha)
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: UI elements scale proportionally with window size

*For any* window size change, all UI elements using percentage-based sizing should maintain their proportional relationships to the window dimensions.
**Validates: Requirements 1.1**

### Property 2: Cover images use UI Node components

*For any* entity with CoverImage1 or CoverImage2 marker components, that entity should have Node and ImageNode components and should not have a Sprite component.
**Validates: Requirements 1.2, 3.4**

### Property 3: Cover images fill entire window

*For any* entity with CoverImage1 or CoverImage2 marker, the Node component should have width and height set to Val::Percent(100.0) and position_type set to PositionType::Absolute.
**Validates: Requirements 1.3**

### Property 4: Animation system queries BackgroundColor not Sprite

*For any* execution of the cover_fade_animation system, the system should query BackgroundColor components and should not query Sprite components.
**Validates: Requirements 2.1**

### Property 5: Smoothstep easing function applied correctly

*For any* cycle progress value between 0.0 and 1.0, the calculated alpha value should follow the smoothstep formula: alpha * alpha * (3.0 - 2.0 * alpha).
**Validates: Requirements 2.3**

### Property 6: Z-axis layering maintained

*For any* menu setup, CoverImage2 entity should have a higher ZIndex value than CoverImage1 entity.
**Validates: Requirements 2.4**

### Property 7: Complementary fade pattern preserved

*For any* point in the animation cycle, when CoverImage1's alpha increases, CoverImage2's alpha should decrease by a corresponding amount, maintaining the complementary relationship.
**Validates: Requirements 2.5**

### Property 8: Menu entities use Node components

*For any* entity with MenuUI marker component, that entity should have a Node component, confirming use of Bevy's UI system.
**Validates: Requirements 3.1**

### Property 9: Button interactions update BackgroundColor

*For any* button entity, when its Interaction component changes state (None/Hovered/Pressed), the BackgroundColor component should update to reflect the new state.
**Validates: Requirements 4.2**

### Property 10: Cleanup removes all menu entities

*For any* execution of cleanup_menu, after the system runs, no entities with MenuUI component should exist in the world.
**Validates: Requirements 4.5**

## Error Handling

### Component Query Failures

如果动画系统无法找到预期的组件，应该优雅地处理：

```rust
pub fn cover_fade_animation(
    mut cover_query: Query<
        (&mut BackgroundColor, &mut CoverFadeState),
        Or<(With<CoverImage1>, With<CoverImage2>)>,
    >,
    time: Res<Time>,
) {
    if cover_query.is_empty() {
        // 没有封面图片时静默返回
        return;
    }
    
    // 正常处理动画逻辑
    // ...
}
```

### 资源加载失败

当GameAssets资源未加载时，系统应该创建简单的颜色背景作为后备：

```rust
if let Some(ref assets) = game_assets {
    // 创建带图片的封面
} else {
    // 创建纯色背景
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2)),
        MenuUI,
    ));
}
```

### 窗口大小边界情况

UI系统应该能够处理极小或极大的窗口尺寸，Bevy的UI系统会自动处理这些情况，但我们需要确保：

- 最小窗口尺寸下UI元素仍然可见
- 文本不会被截断到不可读
- 按钮保持可点击的最小尺寸

## Testing Strategy

### Unit Testing

单元测试将验证特定的组件行为和系统逻辑：

1. **组件类型验证测试**: 验证setup_menu创建的实体具有正确的组件组合
2. **动画计算测试**: 测试smoothstep函数和alpha计算的正确性
3. **清理测试**: 验证cleanup_menu正确移除所有MenuUI实体
4. **按钮交互测试**: 测试特定按钮点击触发正确的状态转换

### Property-Based Testing

属性测试将使用 **proptest** 库（Rust的标准PBT库）来验证系统在各种输入下的正确性：

1. **窗口尺寸属性**: 生成随机窗口尺寸，验证UI元素的比例关系
2. **动画周期属性**: 生成随机时间点，验证alpha值的计算和互补关系
3. **组件查询属性**: 验证所有CoverImage实体都有正确的组件类型
4. **Z-index属性**: 验证层级顺序在所有情况下都正确

**配置要求**:
- 每个属性测试至少运行100次迭代
- 每个属性测试必须使用注释标记对应的设计文档属性编号
- 标记格式: `// Feature: menu-ui-refactor, Property {number}: {property_text}`

### Integration Testing

集成测试将验证整个菜单系统的端到端行为：

1. 完整的菜单初始化和清理流程
2. 动画系统与UI系统的集成
3. 按钮交互与状态管理的集成
4. 资源加载与UI渲染的集成

### Test Organization

```
src/
  systems/
    menu.rs              # 实现代码
    menu.test.rs         # 单元测试（如果需要）
  tests/
    menu_ui_tests.rs     # 集成测试和属性测试
```

## Implementation Notes

### 关键修改点

1. **setup_menu函数** (~50行修改):
   - 将Sprite组件替换为Node + ImageNode
   - 添加ZIndex组件明确层级
   - 使用Val::Percent(100.0)替代固定尺寸
   - 调整交互层的ZIndex确保在封面之上

2. **cover_fade_animation函数** (~30行修改):
   - 修改查询类型从`Query<&mut Sprite>` 到 `Query<&mut BackgroundColor>`
   - 使用`background_color.0.set_alpha()`替代`sprite.color.set_alpha()`
   - 保持相同的动画逻辑和计算

3. **组件定义** (~20行修改):
   - 可能需要在components/ui.rs中添加文档说明组件现在用于UI节点
   - 不需要修改组件结构本身

### 迁移步骤

1. 首先修改setup_menu中的封面图片创建逻辑
2. 然后修改cover_fade_animation的查询和更新逻辑
3. 调整交互层UI的ZIndex确保正确显示
4. 测试不同窗口尺寸下的显示效果
5. 验证所有按钮交互仍然正常工作

### 性能考虑

- UI节点的布局计算由Bevy引擎优化处理，性能应该与Sprite相当或更好
- 动画系统的性能不会受到影响，因为只是修改了查询的组件类型
- 响应式布局在窗口大小改变时会触发重新计算，但这是Bevy自动处理的

### 兼容性

- 此重构不影响其他游戏系统
- 保存/加载系统不需要修改
- 游戏状态管理保持不变
- 其他UI系统（如暂停菜单）可以参考此实现进行类似重构
