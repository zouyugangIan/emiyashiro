---
inclusion: always
---

# Rust Bevy Game Development Standards

## Code Organization

### Module Structure
- 使用清晰的模块层次结构
- 将组件、系统、资源分别组织到独立模块
- 使用 `mod.rs` 文件管理模块导出

### Naming Conventions
- 组件使用 PascalCase：`Player`, `Velocity`, `Health`
- 系统函数使用 snake_case：`player_movement`, `handle_collisions`
- 常量使用 SCREAMING_SNAKE_CASE：`GRAVITY`, `JUMP_FORCE`
- 资源类型使用 PascalCase：`GameState`, `ScoreResource`

## Bevy Best Practices

### Component Design
- 优先使用小而专一的组件
- 避免在组件中存储复杂的业务逻辑
- 使用标记组件（marker components）来标识特殊实体

### System Design
- 保持系统函数简洁和专一
- 使用查询过滤器提高性能
- 合理使用系统参数的顺序（Commands 通常放在最前面）

### Query Optimization
- 使用 `With` 和 `Without` 过滤器优化查询
- 避免在单个系统中进行过多的查询
- 使用 `Changed` 过滤器只处理变化的组件

### Error Handling
- 使用 `single()` 和 `single_mut()` 替代已弃用的方法
- 优雅处理查询失败的情况
- 使用 Result 类型处理可能失败的操作

## Performance Guidelines

### Memory Management
- 避免频繁的实体创建和销毁
- 使用对象池模式管理临时实体
- 合理使用 `Commands` 延迟执行变更

### System Scheduling
- 使用系统集合（SystemSet）组织相关系统
- 明确定义系统间的依赖关系
- 利用并行执行提高性能

## Code Quality

### Documentation
- 为所有公共 API 提供文档注释
- 使用示例代码说明复杂的用法
- 保持文档与代码同步更新

### Testing
- 为核心逻辑编写单元测试
- 使用集成测试验证系统协作
- 测试边界条件和错误情况

### Code Style
- 遵循 Rust 官方代码风格指南
- 使用 `cargo fmt` 格式化代码
- 修复所有 `cargo clippy` 警告

## Version Compatibility

### Bevy Version Management
- 明确指定 Bevy 版本依赖
- 及时更新到稳定版本
- 注意 API 变更和迁移指南

### API Migration
- 使用最新的 API 替代已弃用的方法
- 关注 Bevy 官方迁移指南
- 测试 API 变更对现有代码的影响