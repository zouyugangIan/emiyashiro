# Requirements Document

## Introduction

本文档定义了将游戏主菜单系统从基于Sprite的实现重构为基于Bevy UI节点的响应式实现的需求。当前菜单使用固定尺寸的Sprite来显示封面图片，无法适应不同的窗口大小。重构后的系统将使用UI节点实现完全响应式的布局，支持任意窗口大小，并保持封面渐变动画效果。

## Glossary

- **MenuSystem**: 主菜单系统，负责显示游戏启动界面和处理用户交互
- **Sprite**: Bevy的2D精灵组件，用于渲染固定尺寸的图像
- **UI Node**: Bevy的UI系统节点，支持响应式布局和百分比尺寸
- **ImageNode**: Bevy UI中用于显示图像的节点组件
- **CoverImage**: 菜单背景封面图片，支持渐变动画效果
- **ResponsiveLayout**: 响应式布局，能够根据窗口大小自动调整UI元素尺寸和位置

## Requirements

### Requirement 1

**User Story:** 作为玩家，我希望菜单界面能够适应不同的窗口大小，这样无论窗口如何调整，菜单都能正确显示。

#### Acceptance Criteria

1. WHEN the game window is resized THEN the MenuSystem SHALL scale all UI elements proportionally to maintain visual consistency
2. WHEN the MenuSystem displays cover images THEN the system SHALL use UI Node components instead of Sprite components
3. WHEN cover images are rendered THEN the system SHALL maintain aspect ratio and fill the entire window background
4. WHEN UI elements are positioned THEN the system SHALL use percentage-based layout values instead of fixed pixel values
5. WHERE the window size changes THEN the MenuSystem SHALL update layout without requiring manual recalculation

### Requirement 2

**User Story:** 作为玩家，我希望封面图片的渐变动画效果在重构后仍然保持流畅自然，这样菜单界面依然具有吸引力。

#### Acceptance Criteria

1. WHEN the cover fade animation runs THEN the system SHALL modify ImageNode color properties instead of Sprite color properties
2. WHEN transitioning between cover images THEN the system SHALL maintain the smooth 15-second cycle duration
3. WHEN calculating alpha values THEN the system SHALL apply the same smoothstep easing function to ensure gradual transitions
4. WHEN both cover images are displayed THEN the system SHALL maintain proper Z-axis layering using UI node hierarchy
5. WHEN animation updates occur THEN the system SHALL preserve the complementary fade pattern between the two images

### Requirement 3

**User Story:** 作为开发者，我希望重构后的代码符合Bevy最佳实践，这样系统更易于维护和扩展。

#### Acceptance Criteria

1. WHEN implementing UI components THEN the MenuSystem SHALL use Bevy's UI node system as the primary layout mechanism
2. WHEN organizing component definitions THEN the system SHALL maintain clear separation between UI components and game components
3. WHEN structuring the menu hierarchy THEN the system SHALL use parent-child relationships to manage Z-ordering
4. WHEN accessing UI properties THEN the system SHALL use appropriate Bevy UI component types (ImageNode, BackgroundColor, etc.)
5. WHERE future UI features are added THEN the system SHALL support extension without requiring architectural changes

### Requirement 4

**User Story:** 作为开发者，我希望重构过程能够保持现有功能完整性，这样不会破坏已有的菜单交互逻辑。

#### Acceptance Criteria

1. WHEN the refactor is complete THEN the MenuSystem SHALL preserve all existing button interactions (Start, Load, Character Select)
2. WHEN users interact with buttons THEN the system SHALL maintain the same visual feedback and state transitions
3. WHEN the menu is initialized THEN the system SHALL load and display the same cover images and UI elements
4. WHEN transitioning to other game states THEN the system SHALL use the same state management logic
5. WHEN cleaning up the menu THEN the system SHALL properly despawn all UI entities without memory leaks
