# 🔄 接替工作触发提示词

## 使用说明

当上下文窗口中断后，将以下提示词发送给新的 Claude Code 会话以接替工作。

---

## 极简版本（推荐）

```
请接手 Claude Config Manager 项目开发。

项目位置: C:\Users\serow\Desktop\cc-workspaces\claude-config-manager

当前状态:
- Phase 1-6 完成 (152/175 任务, 87%)
- 207 个测试全部通过
- MVP 核心功能已完整 (CLI + GUI)
- GUI 后端命令已实现并注册

上一次工作:
修复了 GUI 后端的生命周期问题，完成了 Tauri 命令注册和构建验证。

请先阅读:
1. docs/handoff/PROJECT_STATUS.md - 项目状态总览
2. docs/reports/ - 最新进度报告
3. docs/handoff/LLM_HANDOVER.md - 详细接续指南

然后继续工作。
```

---

## 详细版本

```
请接手 Claude Config Manager 项目开发工作。

📍 项目位置
C:\Users\serow\Desktop\cc-workspaces\claude-config-manager

📊 当前状态
- Phase: 6/12 完成 (50%)
- 任务: 152/175 完成 (87%)
- 测试: 207 个，100% 通过
- 编译: ✅ 0 错误
- MVP: ✅ 功能完整

✅ 已完成
- Phase 1-6: 核心功能
- CLI: config + mcp + history 命令
- GUI: Tauri 后端命令实现
- GUI: React 前端界面
- 测试: 207 个单元测试

🎯 最近工作 (2025-01-21)
- 修复 GUI 后端生命周期错误
- 注册所有 Tauri 命令 (mcp + history)
- 前后端构建验证成功
- 详见: docs/reports/GUI_IMPLEMENTATION_REPORT.md

📋 下一步工作
请选择:
1. 测试 GUI 应用功能
2. 实现剩余高级功能 (Phase 7-10)
3. 准备发布 (Phase 11-12)

📖 参考文档
优先阅读:
1. docs/handoff/PROJECT_STATUS.md - 快速状态
2. docs/handoff/LLM_HANDOVER.md - 详细指南
3. docs/reports/ - 最新进度报告

然后基于当前状态继续推进项目。
```

---

## 快速命令版本

```
cd C:\Users\serow\Desktop\cc-workspaces\claude-config-manager

请阅读 docs/handoff/PROJECT_STATUS.md 了解项目状态，
然后继续上次的工作。

上次工作: 修复 GUI 后端，详见 docs/reports/GUI_IMPLEMENTATION_REPORT.md
```

---

## 项目路径引用

如果需要直接引用文件：

```
项目根目录: C:\Users\serow\Desktop\cc-workspaces\claude-config-manager

关键文档:
- 项目状态: docs/handoff/PROJECT_STATUS.md
- 接续指南: docs/handoff/LLM_HANDOVER.md
- 最新报告: docs/reports/ (读取最新的 .md 文件)
- 任务列表: specs/001-initial-implementation/tasks.md

关键目录:
- 核心库: crates/core/src/
- CLI: crates/cli/src/
- GUI: ui/src/ (前端) + ui/src-tauri/src/ (后端)
```

---

## 状态检查命令

如果需要快速检查当前状态：

```bash
# 检查项目状态
cat docs/handoff/PROJECT_STATUS.md

# 检查最新报告
ls -lt docs/reports/*.md | head -5

# 检查构建状态
cd ui && npm run build        # 前端
cd ui/src-tauri && cargo build # 后端

# 运行测试
cargo test --workspace
```

---

## 常见接续场景

### 场景 1: 继续功能开发
```
项目: Claude Config Manager
位置: C:\Users\serow\Desktop\cc-workspaces\claude-config-manager
状态: Phase 1-6 完成，GUI 已就绪

请实现 Phase 7: 项目发现和扫描功能。
参考: specs/001-initial-implementation/tasks.md
```

### 场景 2: 修复 Bug
```
项目: Claude Config Manager
位置: C:\Users\serow\Desktop\cc-workspaces\claude-config-manager

GUI 应用无法启动，请诊断问题。
错误信息: [粘贴错误]
```

### 场景 3: 准备发布
```
项目: Claude Config Manager
位置: C:\Users\serow\Desktop\cc-workspaces\claude-config-manager
状态: MVP 功能完成

请准备 Phase 11-12: QA 质量保证和发布准备。
```

---

## 提示词设计原则

1. **简洁优先** - 只包含必要信息
2. **路径明确** - 提供完整项目路径
3. **状态清晰** - 当前进度和最近工作
4. **文档指引** - 指向详细参考文档
5. **行动明确** - 告诉下一步做什么

---

**文件维护**: 当项目有重大进展时，请更新此文件。
**最后更新**: 2025-01-21
**当前版本**: v1.0
