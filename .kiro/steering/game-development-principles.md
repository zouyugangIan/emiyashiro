---
inclusion: always
---

# Game Development Principles

## Core Design Philosophy

### Player Experience First
- 优先考虑玩家的直观体验
- 确保操作响应及时且可预测
- 提供清晰的视觉和音频反馈

### Iterative Development
- 从最小可玩原型开始
- 快速迭代和测试核心机制
- 基于反馈持续改进游戏体验

## Game Architecture

### Entity Component System (ECS)
- 使用组合而非继承设计游戏对象
- 保持组件数据和系统逻辑分离
- 利用 ECS 的性能优势和灵活性

### State Management
- 明确定义游戏状态和状态转换
- 使用状态机管理复杂的游戏流程
- 确保状态变化的一致性和可预测性

## Performance Considerations

### Frame Rate Stability
- 目标帧率：60 FPS
- 避免帧率波动影响游戏体验
- 使用性能分析工具识别瓶颈

### Memory Efficiency
- 避免频繁的内存分配和释放
- 使用对象池管理临时对象
- 监控内存使用情况

## User Interface Design

### Accessibility
- 支持多种输入方式（键盘、手柄等）
- 提供可调节的游戏设置
- 考虑不同能力玩家的需求

### Feedback Systems
- 提供即时的操作反馈
- 使用视觉、音频、触觉反馈
- 确保反馈的一致性和清晰性

## Quality Assurance

### Testing Strategy
- 单元测试：测试核心逻辑
- 集成测试：验证系统协作
- 用户测试：收集真实反馈

### Bug Prevention
- 使用类型系统防止常见错误
- 实现全面的错误处理
- 定期进行代码审查

## Maintainability

### Code Organization
- 使用清晰的模块结构
- 保持代码的可读性和可维护性
- 编写自文档化的代码

### Configuration Management
- 将游戏参数外部化配置
- 支持运行时调整参数
- 使用版本控制管理配置变更

## Localization and Accessibility

### Multi-language Support
- 设计支持多语言的文本系统
- 考虑不同语言的文本长度差异
- 提供语言切换功能

### Platform Compatibility
- 确保在目标平台上的兼容性
- 适配不同的屏幕尺寸和分辨率
- 优化不同平台的性能表现