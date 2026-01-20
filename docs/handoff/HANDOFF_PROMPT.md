# Claude Config Manager - Context Handoff Prompt

**Last Updated**: 2025-01-19
**Project Location**: `C:\Users\serow\Desktop\cc-workspaces\claude-config-manager`
**Git Repository**: https://github.com/unnode001/claude-config-manager

---

## 📋 Current Status Summary

### Progress: Phase 2 ~80% Complete (25/31 tasks)
- ✅ **Phase 1**: Project Setup (T001-T012) - 100% Complete
- ✅ **Phase 2.1**: Configuration Types (T013-T018) - Complete
- ✅ **Phase 2.2**: Error Handling (T019-T021) - Complete
- ✅ **Phase 2.3**: Configuration Validation (T022-T026) - Complete
- ✅ **Phase 2.4**: Backup System (T027-T030) - Complete
- ✅ **Phase 2.5**: Configuration File I/O (T031-T034) - Complete
- 🔄 **Phase 2.6**: Configuration Merging (T035-T039) - **IN PROGRESS**
- ⏸️ **Phase 2.7**: Path Handling (T040-T043) - Pending

### Test Status: **72 tests passing ✅**
- 46 unit tests (all passing)
- 26 integration tests (all passing)
- 100% TDD compliance

---

## 🎯 Immediate Next Task

**You are working on**: T035-T039 - Configuration Merging

### Current Todo Status:
```
T035: Create merge.rs module structure - IN PROGRESS
T036: Implement deep merge for objects - PENDING
T037: Implement replace strategy for arrays/primitives - PENDING
T038: Write unit tests for merge behavior - PENDING
T039: Write integration tests for multi-level merging - PENDING
```

### What to Do Next:

1. **Create** `crates/core/src/config/merge.rs` module
2. **Follow TDD** - Write tests FIRST (T038), then implement (T036-T037)
3. **Implement** `merge_configs()` function with:
   - Deep merge for objects (nested structures merge recursively)
   - Replace strategy for arrays (higher scope replaces lower scope)
   - Replace strategy for primitives (higher scope replaces lower scope)
   - Source tracking (optional SourceMap for debugging)
4. **Add** merge module to `config/mod.rs` exports
5. **Write** integration tests in `crates/core/tests/merge_integration.rs` (T039)

---

## 📁 Key Files to Reference

### Core Implementation Files:
```
crates/core/src/
├── lib.rs                      # Re-exports, module declarations
├── error.rs                    # ConfigError enum (7 variants)
├── types.rs                    # Shared types (McpServer, Skill, ConfigScope, etc.)
├── config/
│   ├── mod.rs                  # ClaudeConfig struct, builder pattern
│   ├── validation.rs           # ValidationRule trait, 3 validation rules
│   ├── manager.rs              # ConfigManager (read/write with backup)
│   └── merge.rs                # [TO CREATE] merge_configs()
├── backup/
│   └── mod.rs                  # BackupManager (create/list/cleanup backups)
```

### Test Files:
```
crates/core/tests/
├── error_messages.rs           # 10 integration tests ✅
├── backup_integration.rs       # 9 integration tests ✅
├── file_io_integration.rs      # 7 integration tests ✅
└── merge_integration.rs        # [TO CREATE] multi-level merge tests
```

### Documentation:
```
specs/001-initial-implementation/
├── spec.md                     # Feature specification
├── plan.md                     # Implementation plan
├── data-model.md               # Data model documentation
├── tasks.md                    # Detailed task breakdown
└── contracts/
    └── claude-config-spec.md   # Claude Config format specification
```

---

## 🔑 Technical Implementation Guidelines

### Configuration Merge Requirements:

From `contracts/claude-config-spec.md` and `plan.md`:

1. **Merge Strategy**:
   - **Objects (nested structures)**: Deep merge
     - Example: `{ "mcpServers": { "npx": {...} } }` merges recursively
     - Lower scope values are overridden by higher scope values for same keys
     - New keys from higher scope are added
   - **Arrays**: Replace (not merge)
     - Example: `allowedPaths: ["~/projects"]` completely replaces lower scope
   - **Primitives**: Replace (not merge)
     - Example: `customInstructions: ["Be concise"]` completely replaces

2. **Configuration Hierarchy** (from lowest to highest priority):
   ```
   Global (~/.claude/config.json)
   └── Project (<project>/.claude/config.json)
       └── Session (in-memory override)
   ```

3. **Function Signature** (from T035):
   ```rust
   pub fn merge_configs(base: &ClaudeConfig, override: &ClaudeConfig) -> ClaudeConfig
   ```

4. **Source Tracking** (optional, for debugging):
   - May include SourceMap to track which config layer contributed each value
   - Helpful for debugging merge issues

### TDD Process (Strict Requirement):

1. **RED Phase** - Write failing tests first
2. **GREEN Phase** - Implement minimum code to pass tests
3. **REFACTOR Phase** - Clean up code while keeping tests green

**Example test-first approach:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // TDD Test: Deep merge objects
    #[test]
    fn test_deep_merge_objects() {
        let base = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]));

        let override_config = ClaudeConfig::new()
            .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]));

        let merged = merge_configs(&base, &override_config);

        // Should have both servers
        assert!(merged.mcp_servers.is_some());
        assert_eq!(merged.mcp_servers.unwrap().len(), 2);
    }

    // TDD Test: Arrays replace (not merge)
    #[test]
    fn test_arrays_replace() {
        let base = ClaudeConfig::new()
            .with_allowed_path("~/projects/base");

        let override_config = ClaudeConfig::new()
            .with_allowed_path("~/projects/override");

        let merged = merge_configs(&base, &override_config);

        // Should only have override path (not both)
        assert_eq!(merged.allowed_paths.unwrap().len(), 1);
        assert_eq!(merged.allowed_paths.unwrap()[0], "~/projects/override");
    }
}
```

### Module Integration:

After implementing `merge.rs`, update `crates/core/src/config/mod.rs`:

```rust
pub mod manager;
pub mod merge;        // ADD THIS
pub mod validation;

// Re-export for convenience
pub use merge::merge_configs;  // ADD THIS
```

Then update `crates/core/src/lib.rs`:

```rust
pub use config::{ClaudeConfig, manager::ConfigManager, merge_configs};
```

---

## 🚨 Common Pitfalls & Solutions

### Pitfall 1: Forgetting to Handle Unknown Fields
**Issue**: When merging, unknown fields (future features) may be lost
**Solution**: Use `#[serde(flatten)]` field properly, merge unknown HashMaps

### Pitfall 2: Incorrect Merge Strategy for Arrays
**Issue**: Accidentally merging arrays instead of replacing
**Solution**: Arrays should ALWAYS replace (higher scope wins completely)

### Pitfall 3: Not Cloning Values Properly
**Issue**: Borrow checker issues when merging
**Solution**: Clone values from base config before modifying:
```rust
let mut merged = base.clone();
// Now modify merged with override values
```

### Pitfall 4: Breaking Existing Tests
**Issue**: New code breaks 72 passing tests
**Solution**: Run `cargo test` frequently, fix regressions immediately

---

## 🧪 Testing Commands

### Run All Tests:
```bash
cd C:\Users\serow\Desktop\cc-workspaces\claude-config-manager
cargo test --workspace
```

### Run Only Merge Tests (once implemented):
```bash
cargo test --package claude-config-manager-core --lib config::merge
```

### Run Integration Tests:
```bash
cargo test --package claude-config-manager-core --test merge_integration
```

### Check Code Quality:
```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

---

## 📊 Constitution Compliance Checklist

### Principle IV: TDD ✅ (MUST MAINTAIN)
- [ ] Write tests BEFORE implementation
- [ ] Follow Red-Green-Refactor cycle
- [ ] Every public function must have tests

### Principle III: Safety and Reliability ✅ (MUST MAINTAIN)
- [ ] No unsafe Rust unless absolutely necessary
- [ ] Handle all Result types properly
- [ ] Provide clear error messages

### Principle I: Core Library First ✅
- [ ] All business logic in `crates/core`
- [ ] Frontend-agnostic implementation

### Principle VIII: Cross-Platform ✅
- [ ] Use `std::path::Path` for all path operations
- [ ] Test on Windows/macOS/Linux via CI

---

## 🎁 Success Criteria for T035-T039

You've completed Configuration Merging when:

1. ✅ `merge_configs()` function exists in `crates/core/src/config/merge.rs`
2. ✅ Deep merge works for objects (mcpServers, skills)
3. ✅ Replace works for arrays (allowedPaths, customInstructions)
4. ✅ Replace works for primitives
5. ✅ Unit tests cover all merge scenarios (≥10 tests)
6. ✅ Integration tests cover multi-level merging (≥5 tests)
7. ✅ All existing 72 tests still pass
8. ✅ Module is exported from `lib.rs`
9. ✅ Code follows rustfmt and clippy guidelines
10. ✅ TDD process was followed (tests written first)

---

## 🚀 After T035-T039 Complete

Move to **T040-T043: Path Handling**:

1. Use `dirs` crate to resolve config paths:
   - Windows: `%APPDATA%\claude\config.json`
   - macOS: `~/Library/Application Support/Claude/config.json`
   - Linux: `~/.config/claude/config.json`

2. Implement project detection:
   - Search upward for `.claude/config.json`
   - Stop at filesystem root or Git repository root

3. Write path resolution tests for all platforms

---

## 📞 Additional Resources

### Spec Documents:
- Full spec: `specs/001-initial-implementation/spec.md`
- Tasks detail: `specs/001-initial-implementation/tasks.md`
- Data model: `specs/001-initial-implementation/data-model.md`

### Progress Reports:
- Phase 2 complete: `PHASE2_COMPLETE_REPORT.md`
- This handoff: `HANDOFF_PROMPT.md`

### Key Code Sections to Understand:

**ClaudeConfig Structure** (`config/mod.rs:18-48`):
- Has mcpServers, allowedPaths, skills, customInstructions
- All fields are Optional
- Has `unknown: HashMap<String, serde_json::Value>` for forward compatibility

**ConfigManager** (`config/manager.rs:22-178`):
- `read_config()` - reads and parses JSON with detailed error messages
- `write_config_with_backup()` - writes with automatic backup and validation
- Uses atomic write pattern (temp file + rename)

**Validation System** (`config/validation.rs:1-176`):
- `ValidationRule` trait
- Three rules: McpServersRule, AllowedPathsRule, SkillsRule
- `validate_config()` runs all rules

---

## 💬 Start Message for New Session

**Copy-paste this to continue work:**

```
I'm continuing work on the claude-config-manager project. I'm currently implementing T035-T039 (Configuration Merging).

The project is at Phase 2 ~80% complete with 72 tests passing. I need to:

1. Create crates/core/src/config/merge.rs with merge_configs() function
2. Follow TDD - write tests first, then implement
3. Implement deep merge for objects, replace for arrays/primitives
4. Write unit tests and integration tests
5. Export the module and verify all tests pass

Please read HANDOFF_PROMPT.md for full context and continue from there.
```

---

**Last Action Before Context Switch**: Created this handoff document
**Next Action**: Implement merge.rs module following TDD principles

---

## ✅ Verification Checklist

When resuming, verify:
- [ ] Read HANDOFF_PROMPT.md
- [ ] Checked current test count (should be 72 passing)
- [ ] Verified TodoWrite status (T035 should be in_progress)
- [ ] Ran `cargo test --workspace` to ensure baseline
- [ ] Read tasks.md T035-T039 for detailed requirements
- [ ] Ready to write tests first (TDD Red phase)

**Good luck! 🚀**
