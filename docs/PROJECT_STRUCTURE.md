# 📁 Claude Config Manager - 项目结构

**更新时间**: 2025-01-21
**版本**: v0.1.0
**状态**: ✅ CLI + GUI 完成

---

## 📂 根目录结构

```
claude-config-manager/
├── crates/                    # 📦 源代码（核心）
│   ├── core/                 # 核心库
│   ├── cli/                  # CLI应用
│   └── tauri/                # GUI (已迁移到 ui/)
│
├── ui/                        # 🖥️ GUI应用 (新增)
│   ├── src/                  # React前端
│   └── src-tauri/            # Tauri后端
│
├── docs/                      # 📚 文档
│   ├── handoff/              # 项目接续文档
│   │   ├── PROJECT_STATUS.md
│   │   ├── CONTEXT_HANDOVER_PROMPT.md  (新增)
│   │   ├── LLM_HANDOVER.md
│   │   └── ...
│   ├── reports/              # 进度报告
│   │   ├── GUI_IMPLEMENTATION_REPORT.md  (最新)
│   │   ├── GUI_HANDOVER_SUMMARY.md       (新增)
│   │   ├── PHASE6_COMPLETION_REPORT.md
│   │   └── ...
│   └── PROJECT_STRUCTURE.md  # 本文件
│
├── specs/                     # 📋 规格文档
│   └── 001-initial-implementation/
│       └── tasks.md
│
├── .github/                   # 🔄 GitHub配置
│   └── workflows/
│       └── ci.yml
│
├── Cargo.toml                 # 📦 工作空间配置
├── Cargo.lock                 # 🔒 依赖锁定
├── clippy.toml                # 🔍 Clippy配置
├── rustfmt.toml               # 🎨 代码格式配置
├── Makefile                   # 🔨 构建脚本
├── LICENSE                    # ⚖️ MIT许可证
├── README.md                  # 📖 项目说明
├── CHANGELOG.md               # 📝 变更日志
├── CONTRIBUTING.md            # 🤝 贡献指南
├── ARCHITECTURE.md            # 🏗️ 架构文档
└── .gitignore                 # 🚫 Git忽略规则
```

---

## 📦 源代码目录 (crates/)

### crates/core/
核心功能库，包含所有业务逻辑：

```
crates/core/
├── src/
│   ├── backup/               # 备份系统
│   │   └── mod.rs           # BackupManager
│   ├── config/              # 配置管理
│   │   ├── manager.rs       # ConfigManager
│   │   ├── merge.rs         # 配置合并
│   │   └── validation.rs    # 配置验证
│   ├── mcp/                 # MCP服务器管理
│   │   ├── manager.rs       # McpManager
│   │   └── mod.rs
│   ├── paths.rs             # 路径处理
│   ├── types.rs             # 共享类型
│   ├── error.rs             # 错误类型
│   └── lib.rs               # 库入口
└── tests/                   # 集成测试
    ├── backup_integration.rs
    ├── validation_integration.rs
    ├── atomic_write_integration.rs
    └── ...
```

### crates/cli/
命令行界面：

```
crates/cli/
├── src/
│   ├── main.rs              # CLI入口
│   ├── commands/            # 命令实现
│   │   ├── config.rs        # config命令
│   │   ├── mcp.rs           # mcp命令
│   │   ├── history.rs       # history命令
│   │   └── mod.rs
│   ├── key_path.rs          # 键路径解析
│   └── output/              # 输出格式
│       ├── json.rs
│       ├── table.rs
│       └── mod.rs
└── tests/
    └── cli_integration.rs   # CLI集成测试
```

### crates/tauri/
GUI应用（已迁移到 ui/）：

```
crates/tauri/  →  ui/src-tauri/
├── src/
│   ├── commands/           # Tauri命令实现
│   ├── lib.rs              # Tauri应用入口
│   └── main.rs             # 主函数
├── tauri.conf.json         # Tauri配置
└── build.rs                # 构建脚本
```

### ui/ (新增)
完整的GUI应用：

```
ui/
├── src/                    # React前端
│   ├── App.tsx            # 主应用组件
│   ├── App.css            # 样式
│   ├── main.tsx           # 入口
│   └── assets/            # 资源
│
├── src-tauri/             # Tauri后端
│   ├── src/
│   │   ├── commands/      # Tauri命令
│   │   │   ├── config.rs  # 配置管理
│   │   │   ├── mcp.rs     # MCP服务器
│   │   │   ├── history.rs # 备份历史
│   │   │   ├── project.rs # 项目管理
│   │   │   ├── search.rs  # 配置搜索
│   │   │   ├── types.rs   # 数据类型
│   │   │   └── utils.rs   # 工具函数
│   │   ├── lib.rs         # Tauri入口
│   │   └── main.rs        # 主函数
│   ├── Cargo.toml         # Rust依赖
│   └── tauri.conf.json    # Tauri配置
│
├── package.json           # Node依赖
├── vite.config.ts         # Vite配置
└── tsconfig.json          # TypeScript配置
```

---

## 📚 文档目录 (docs/)

### docs/handoff/
项目接续和状态文档：

- **PROJECT_STATUS.md** - 项目状态总览 ⭐
- **CONTEXT_HANDOVER_PROMPT.md** - 极简接替提示词 (新增) ⭐
- **LLM_HANDOVER.md** - LLM接续指南
- **HANDOVER_PROMPT.md** - 手动接续提示

### docs/reports/
各阶段完成报告：

- **GUI_IMPLEMENTATION_REPORT.md** - GUI实施详细报告 (最新) ⭐
- **GUI_HANDOVER_SUMMARY.md** - GUI工作总结 (新增)
- **PHASE6_COMPLETION_REPORT.md** - Phase 6完成报告
- **TESTING_OPTIMIZATION_REPORT.md** - 测试优化报告
- **WORK_SUMMARY_REPORT.md** - 工作总结
- **PROJECT_PROGRESS_REPORT.md** - 项目进度报告

### docs/PROJECT_STRUCTURE.md
本文件 - 项目结构说明

---

## ⚙️ 配置文件

| 文件 | 用途 |
|------|------|
| **Cargo.toml** | 工作空间配置 |
| **Cargo.lock** | 依赖版本锁定 |
| **clippy.toml** | Rust linter配置 |
| **rustfmt.toml** | 代码格式化配置 |
| **.gitignore** | Git忽略规则 |
| **LICENSE** | MIT许可证 |

---

## 📋 specs/ 目录

规格和任务文档：

```
specs/
└── 001-initial-implementation/
    └── tasks.md              # 完整的175个任务列表
```

---

## 🎯 使用指南

### 开发者
```bash
# 核心开发
cd crates/core/          # 核心库开发
cd crates/cli/           # CLI开发
cd ui/                   # GUI开发

# 测试
cargo test               # 运行所有测试
cargo test --lib        # 单元测试
cargo test --test *_integration  # 集成测试

# 构建CLI
cargo build --bin ccm
cargo build --release

# 构建GUI
cd ui && npm run build          # 前端
cd ui/src-tauri && cargo build # 后端
cd ui && npm run tauri build    # 完整应用
```

### 用户
```bash
# 运行CLI
cargo run --bin ccm -- --help
ccm --help                # 查看帮助
ccm config get           # 查看配置
ccm mcp list             # 列出MCP服务器
ccm history list         # 查看备份

# 运行GUI
cd ui
npm run tauri dev        # 开发模式
npm run tauri build      # 生产构建
```

---

## 📊 文件统计

| 类型 | 数量 | 位置 |
|------|------|------|
| **核心源代码** | ~20 | crates/core/src/ |
| **CLI源代码** | ~10 | crates/cli/src/ |
| **GUI前端代码** | ~5 | ui/src/ |
| **GUI后端代码** | ~10 | ui/src-tauri/src/ |
| **测试文件** | ~15 | crates/core/tests/, crates/cli/tests/ |
| **配置文件** | 8 | 根目录 |
| **文档文件** | ~20 | docs/ |
| **规格文件** | 1 | specs/ |

---

## 📝 维护原则

1. **源代码放在 `crates/` 或 `ui/` 目录**
   - crates/core/ - 核心库
   - crates/cli/ - CLI应用
   - ui/ - GUI应用

2. **文档放在 `docs/` 目录**
   - docs/handoff/ - 项目接续文档
   - docs/reports/ - 进度报告

3. **规格放在 `specs/` 目录**

4. **配置文件保留在根目录**
   - Cargo.toml, package.json
   - tauri.conf.json
   - .gitignore, clippy.toml, rustfmt.toml

5. **临时文件加入 .gitignore**
   - target/, dist/, node_modules/
   - .backups/, test_project/

---

**最后更新**: 2025-01-21
**项目状态**: 🟢 CLI + GUI 完成
