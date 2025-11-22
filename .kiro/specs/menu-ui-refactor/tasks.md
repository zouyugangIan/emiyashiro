# Implementation Plan

- [x] 1. 重构封面图片为UI节点
  - 修改 `src/systems/menu.rs` 中的 `setup_menu` 函数
  - 将第一张封面图片从Sprite改为Node + ImageNode组件
  - 将第二张封面图片从Sprite改为Node + ImageNode组件
  - 使用 `Val::Percent(100.0)` 设置宽度和高度实现响应式布局
  - 添加 `ZIndex` 组件明确层级顺序（CoverImage1: 0, CoverImage2: 1）
  - 使用 `PositionType::Absolute` 确保封面填充整个窗口
  - 保留后备方案：当资源未加载时创建纯色背景
  - _Requirements: 1.2, 1.3, 3.1_

- [x] 1.1 编写属性测试：验证封面图片使用UI节点组件


  - **Property 2: Cover images use UI Node components**
  - **Validates: Requirements 1.2, 3.4**
  - 测试所有CoverImage实体具有Node和ImageNode组件
  - 测试所有CoverImage实体不具有Sprite组件



- [x] 1.2 编写属性测试：验证封面图片填充整个窗口


  - **Property 3: Cover images fill entire window**
  - **Validates: Requirements 1.3**


  - 测试CoverImage节点的width和height为Val::Percent(100.0)
  - 测试position_type为PositionType::Absolute

- [x] 1.3 编写属性测试：验证Z轴层级正确


  - **Property 6: Z-axis layering maintained**
  - **Validates: Requirements 2.4**
  - 测试CoverImage2的ZIndex高于CoverImage1

- [x] 2. 调整交互层UI的Z-index




  - 修改 `setup_menu` 中UI根节点的创建
  - 为交互层根节点添加 `ZIndex(2)` 确保在封面之上
  - 验证按钮和文本元素正确显示在封面图片上方
  - _Requirements: 3.3_

- [x] 2.1 编写属性测试：验证菜单实体使用Node组件


  - **Property 8: Menu entities use Node components**
  - **Validates: Requirements 3.1**
  - 测试所有MenuUI实体都有Node组件

- [x] 3. 重构封面渐变动画系统
  - 修改 `src/systems/menu.rs` 中的 `cover_fade_animation` 函数
  - 将查询类型从 `Query<&mut Sprite>` 改为 `Query<&mut BackgroundColor>`
  - 使用 `background_color.0.set_alpha()` 替代 `sprite.color.set_alpha()`
  - 保持相同的动画周期（15秒）和smoothstep缓动函数
  - 保持互补的渐变模式（一张淡入时另一张淡出）
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 3.1 编写属性测试：验证动画系统查询正确组件
  - **Property 4: Animation system queries BackgroundColor not Sprite**
  - **Validates: Requirements 2.1**
  - 验证动画系统不查询Sprite组件
  - 验证动画系统查询BackgroundColor组件

- [x] 3.2 编写属性测试：验证smoothstep缓动函数


  - **Property 5: Smoothstep easing function applied correctly**
  - **Validates: Requirements 2.3**
  - 对任意cycle progress值测试alpha计算使用smoothstep公式
  - 公式: alpha * alpha * (3.0 - 2.0 * alpha)



- [x] 3.3 编写属性测试：验证互补渐变模式

  - **Property 7: Complementary fade pattern preserved**
  - **Validates: Requirements 2.5**
  - 测试在动画周期的任意时刻，两张图片的alpha值保持互补关系

- [x] 4. 更新组件文档



  - 修改 `src/components/ui.rs` 中的文档注释
  - 更新 `CoverImage1` 和 `CoverImage2` 的文档说明它们现在用于UI节点
  - 更新 `CoverFadeState` 的文档说明它与BackgroundColor配合使用
  - _Requirements: 3.2_

- [x] 5. 验证按钮交互功能
  - 运行游戏测试所有按钮交互
  - 验证开始按钮、加载按钮、角色选择按钮的点击响应
  - 验证按钮悬停和按下时的视觉反馈
  - 验证状态转换正常工作
  - _Requirements: 4.1, 4.2, 4.4_

- [x] 5.1 编写属性测试：验证按钮交互更新颜色
  - **Property 9: Button interactions update BackgroundColor**
  - **Validates: Requirements 4.2**
  - 测试按钮Interaction状态变化时BackgroundColor更新



- [x] 5.2 编写单元测试：验证按钮点击触发状态转换
  - 测试开始按钮点击触发GameState::Playing转换
  - 测试加载按钮点击触发GameState::LoadTable转换


  - 测试角色选择按钮更新CharacterSelection资源
  - _Requirements: 4.1, 4.4_

- [x] 6. 测试响应式布局
  - 在不同窗口尺寸下测试菜单显示
  - 验证封面图片正确填充窗口

  - 验证UI元素保持居中和正确比例
  - 测试极小和极大窗口尺寸的边界情况

  - _Requirements: 1.1, 1.5_

- [x] 6.1 编写属性测试：验证UI元素比例缩放
  - **Property 1: UI elements scale proportionally with window size**
  - **Validates: Requirements 1.1**


  - 生成随机窗口尺寸测试UI元素保持比例关系

- [x] 7. 验证清理功能
  - 测试从菜单切换到游戏状态时cleanup_menu正确执行
  - 验证所有MenuUI实体被正确移除
  - 确认没有内存泄漏或孤立实体
  - _Requirements: 4.5_

- [x] 7.1 编写属性测试：验证清理移除所有菜单实体
  - **Property 10: Cleanup removes all menu entities**
  - **Validates: Requirements 4.5**
  - 测试cleanup_menu执行后不存在MenuUI实体

- [x] 8. 编写集成测试
  - 测试完整的菜单初始化和清理流程
  - 测试动画系统与UI系统的集成
  - 测试资源加载失败时的后备方案
  - 创建测试文件 `src/tests/menu_ui_tests.rs`
  - _Requirements: 4.3_

- [x] 9. 最终检查点 - 确保所有测试通过
  - 确保所有测试通过，如有问题请询问用户
