# Contributing to Shirou Runner

感谢你对 Shirou Runner 项目的关注！我们欢迎各种形式的贡献。

## 如何贡献

### 报告 Bug

如果你发现了 bug，请创建一个 issue 并包含以下信息：

- 清晰的 bug 描述
- 重现步骤
- 预期行为 vs 实际行为
- 系统环境信息（操作系统、Rust 版本等）
- 相关的错误日志或截图

### 建议新功能

我们欢迎新功能建议！请创建一个 issue 并包含：

- 功能的详细描述
- 使用场景和用户价值
- 可能的实现方案
- 相关的设计草图或原型

### 代码贡献

#### 开发环境设置

1. Fork 本项目
2. 克隆你的 fork：
   ```bash
   git clone https://github.com/your-username/shirou-runner.git
   cd shirou-runner
   ```
3. 安装依赖：
   ```bash
   cargo build
   ```
4. 运行测试：
   ```bash
   cargo test
   ```

#### 开发流程

1. 创建功能分支：
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. 进行开发并遵循项目标准
3. 添加或更新测试
4. 确保所有测试通过：
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. 提交更改：
   ```bash
   git commit -m "feat: add your feature description"
   ```
6. 推送到你的 fork：
   ```bash
   git push origin feature/your-feature-name
   ```
7. 创建 Pull Request

#### 代码标准

请遵循以下代码标准：

- **Rust 代码风格**: 使用 `cargo fmt` 格式化代码
- **Linting**: 修复所有 `cargo clippy` 警告
- **命名规范**: 遵循 [Rust Bevy 开发标准](.kiro/steering/rust-bevy-standards.md)
- **文档**: 为公共 API 添加文档注释
- **测试**: 为新功能添加适当的测试

#### Commit 消息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档更新
- `style:` 代码格式化
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建或辅助工具相关

示例：
```
feat: add player jump animation
fix: resolve camera following lag issue
docs: update API documentation for movement system
```

### Pull Request 指南

#### PR 标题
使用清晰描述性的标题，遵循 commit 消息规范。

#### PR 描述
包含以下信息：

- **变更摘要**: 简要描述你的更改
- **相关 Issue**: 链接相关的 issue（如果有）
- **测试**: 描述你如何测试了这些更改
- **截图**: 如果涉及 UI 变更，请提供截图
- **Breaking Changes**: 如果有破坏性变更，请明确说明

#### PR 检查清单

在提交 PR 前，请确保：

- [ ] 代码遵循项目标准
- [ ] 所有测试通过
- [ ] 添加了适当的测试
- [ ] 更新了相关文档
- [ ] Commit 消息符合规范
- [ ] 没有合并冲突

### 项目规范开发

如果你想参与项目规范的开发，请查看 `.kiro/specs/` 目录下的文档：

- 阅读现有的需求和设计文档
- 按照任务列表中的优先级进行开发
- 确保实现符合需求文档中的验收标准

### 社区准则

- 保持友善和专业的态度
- 尊重不同的观点和经验水平
- 提供建设性的反馈
- 帮助新贡献者融入项目

### 获得帮助

如果你需要帮助：

- 查看项目文档和规范
- 在 issue 中提问
- 参考 [Bevy 官方文档](https://bevyengine.org/learn/)
- 查看 [Rust 官方文档](https://doc.rust-lang.org/)

## 许可证

通过贡献代码，你同意你的贡献将在与项目相同的 MIT 许可证下发布。

---

再次感谢你的贡献！🎮✨