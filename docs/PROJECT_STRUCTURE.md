# 项目目录结构说明

## 📂 根目录结构

```
claude-config-manager/
├── crates/                    # 📦 源代码（核心）
│   ├── core/                 # 核心库
│   ├── cli/                  # CLI应用
│   └── tauri/                # GUI应用（待实现）
│
├── docs/                      # 📚 文档
│   ├── handoff/              # 项目交接文档
│   │   ├── LLM_HANDOVER.md
│   │   ├── HANDOVER_PROMPT.md
│   │   ├── PROJECT_STATUS.md
│   │   └── IMPLEMENTATION_PROGRESS.md
│   ├── reports/              # 进度报告
│   │   ├── PHASE6_COMPLETION_REPORT.md
│   │   ├── TESTING_OPTIMIZATION_REPORT.md
│   │   └── ...
│   └── QUICK_START_GUIDE.md
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
├── LICENSE                    # ⚖️ MIT许可证
├── README.md                  # 📖 项目说明
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
GUI应用（待实现）：

```
crates/tauri/
├── src/
│   ├── lib.rs               # Tauri插件
│   └── main.rs              # GUI入口
├── tauri.conf.json          # Tauri配置
└── build.rs                 # 构建脚本
```

---

## 📚 文档目录 (docs/)

### docs/handoff/
项目交接和状态文档：

- **LLM_HANDOVER.md** - 简洁的LLM接续提示
- **HANDOVER_PROMPT.md** - 详细的接续文档
- **PROJECT_STATUS.md** - 项目状态总览
- **IMPLEMENTATION_PROGRESS.md** - 实施进度

### docs/reports/
各阶段完成报告：

- **PHASE6_COMPLETION_REPORT.md** - Phase 6完成报告
- **TESTING_OPTIMIZATION_REPORT.md** - 测试优化报告
- **WORK_SUMMARY_REPORT.md** - 工作总结
- **PROJECT_PROGRESS_REPORT.md** - 项目进度报告
- ... (其他报告)

### QUICK_START_GUIDE.md
快速开始指南

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
# 工作目录
cd crates/core/          # 核心库开发
cd crates/cli/           # CLI开发

# 测试
cargo test               # 运行所有测试
cargo test --lib        # 单元测试
cargo test --test *_integration  # 集成测试
```

### 用户
```bash
# 运行CLI
cargo run --bin ccm -- --help

# 或安装后
ccm --help
```

---

## 📊 文件统计

| 类型 | 数量 | 位置 |
|------|------|------|
| **源代码文件** | ~20 | crates/ |
| **测试文件** | ~15 | crates/core/tests/, crates/cli/tests/ |
| **配置文件** | 6 | 根目录 |
| **文档文件** | ~15 | docs/ |
| **规格文件** | 1 | specs/ |

---

**维护原则**:
1. 源代码放在 `crates/` 目录
2. 文档放在 `docs/` 目录
3. 规格放在 `specs/` 目录
4. 配置文件保留在根目录
5. 临时文件加入 .gitignore
