# DevKit v0.2.0 版本开发文档

> 版本主题：Smart Formatter & Text Diff
>
> 目标：完成 JSONL 支持、通用文本格式化与文本 Diff，并建立可验证的 GitHub Release 与 Homebrew 发布链路。

## 1. 版本目标

本版本交付以下用户可见能力：

- Json Parser 支持 JSONL，自动转换为 JSON Array 并格式化。
- JSON Diff 升级为通用文本 Diff。
- 自动识别 JSON、TOML、YAML、Rust、JS/TS、Java、C/C++、Lua 和普通文本。
- 对结构化数据和代码进行格式化后再对比。
- 支持应用内文本 Diff，同时保留外部 diff 工具。
- 建立 CI、自动测试、GitHub Release 和 Homebrew Tap 发布流程。
- 完善 README、CHANGELOG、Roadmap 和官网发布资料。

## 2. 状态与任务规则

任务状态：

- `Planned`：已规划，尚未开始
- `In Progress`：正在开发
- `Blocked`：被前置任务或外部条件阻塞
- `Review`：实现完成，等待验证或审查
- `Done`：验收完成

执行规则：

- 任务必须按照依赖关系推进，除非明确记录例外原因。
- 每个任务完成后必须执行该任务的验证命令或验收场景。
- 任务进入 `Done` 前，必须更新本文件状态和结果记录。
- 任何影响公共行为的变更都必须补充测试和 Changelog 条目。
- 任何发布相关任务失败时，不得继续执行正式版本发布。

## 3. 任务总览

| ID | 任务 | 状态 | Blocked by | 验收结果 |
|---|---|---|---|---|
| T01 | 建立版本发布基线 | Done | None | Cargo 为版本基准，npm/Tauri 一致性检查通过 |
| T02 | 建立基础 CI | Done | T01 | CI、sidecar 构建顺序与全部本地质量门禁通过 |
| T03 | 增加 JSON/JSONL 核心解析 | Done | T01, T02 | JSONL 转 JSON Array，非法行错误含行号，测试通过 |
| T04 | Json Parser 接入 JSONL | Done | T03 | UI 显示 JSON/JSONL 识别结果，构建通过 |
| T05 | 增加内容类型识别 | Done | T02 | 目标类型识别、空/未知回退与手动覆盖测试通过 |
| T06 | 增加统一文本格式化 | Done | T03, T05 | JSON/JSONL、TOML、YAML 与文本回退格式化测试通过 |
| T07 | JSON Diff 升级为通用文本 Diff | Done | T05, T06 | JSON、代码与普通文本均可在应用内比较 |
| T08 | 增加应用内文本 Diff UI | Done | T07 | 行级高亮、滚动同步、复制、保存与外部入口可用 |
| T09 | 补齐单元测试与集成测试 | Done | T03, T06, T07 | 核心单测与离线集成测试稳定通过 |
| T10 | 建立 GitHub Release | In Progress | T01, T02, T09 | 开始准备 tag 发布与校验资产工作流 |
| T10 | 建立 GitHub Release | Planned | T01, T02, T09 | tag 可生成跨平台 Release |
| T11 | 增加 Homebrew Tap | In Progress | T10 | Formula 模板与自动更新流程已就绪，待创建 Tap 并验证安装 |
| T12 | 完成文档、官网和正式发布 | Planned | T08, T10, T11 | v0.2.0 可正式发布 |

## 4. 详细执行任务

### T01：建立版本发布基线

**交付内容**

- 统一 Cargo、npm、Tauri 版本号来源。
- 定义 `v0.2.0` 发布检查规则。
- 确认 CLI、GUI、Release 资产命名和平台矩阵。
- 建立 Changelog、README、Roadmap 和 Release 的版本同步约束。

**验收标准**

- 版本信息不一致时检查失败。
- CLI、GUI 和 Release 版本来源明确且可重复验证。
- 发布资产名称和支持平台已记录。

### T02：建立基础 CI

**交付内容**

- Rust fmt、Clippy、单元测试和 workspace build。
- 前端依赖安装和构建。
- 修复 Tauri 构建前必须生成 CLI deployment binary 的顺序问题。
- 配置 Cargo、npm 和构建缓存。
- PR 与 `main` 分支自动检查。

**验收标准**

- PR 自动触发 CI。
- CI 可以稳定完成 Rust 核心检查和前端构建。
- Tauri 构建不再因为缺少 `target/deployment/.../devkit` 失败。

### T03：增加 JSON/JSONL 核心解析

**交付内容**

- 标准 JSON 解析失败后尝试 JSONL。
- JSONL 按非空行解析并转换为 JSON Array。
- 非法行返回行号和解析原因。
- 解析结果继续支持 JSONPath、key/value 查询和格式化。

**验收标准**

- 支持 object、array、string、number、boolean、null 记录。
- 空行默认跳过。
- 非法行立即失败并报告行号。
- 原有标准 JSON 行为不变。

### T04：Json Parser 接入 JSONL

**交付内容**

- Json Parser 输入框支持 JSONL。
- JSONL 结果使用现有 JSON Array 视图展示。
- 保持 tab、查询、保存和复制功能。
- UI 显示 JSON/JSONL 识别结果和错误信息。

**验收标准**

- 粘贴 JSONL 后自动格式化。
- JSONPath 数组查询可用。
- 错误信息包含行号。
- 标准 JSON 输入回归通过。

### T05：增加内容类型识别

**交付内容**

- 支持 JSON、TOML、YAML、Rust、JS/TS、Java、C/C++、Lua 和普通文本。
- 支持手动类型覆盖自动识别。
- 左右输入可以分别识别。
- 自动识别失败时回退普通文本。

**验收标准**

- 每个目标类型都有识别测试。
- 空内容和未知内容不会崩溃。
- 识别结果可传递给格式化和 Diff 流程。

### T06：增加统一文本格式化

**交付内容**

- JSON、TOML、YAML 标准化格式化。
- JSONL 通过 JSON Array 格式化。
- 代码类型提供无外部依赖的基础格式化或安全回退。
- 普通文本统一换行和尾部空白策略。

**验收标准**

- 等价结构化数据输出稳定一致。
- 格式化失败不会阻塞普通文本对比。
- 外部 formatter 不存在时仍可使用基本功能。

### T07：JSON Diff 升级为通用文本 Diff

**交付内容**

- Diff 输入不再强制转换为 JSON。
- 自动识别、格式化左右内容。
- 普通文本作为兜底。
- 保留 JSONPath 查询兼容。
- 保留 IDEA、VSCode、Zed 外部 diff 工具。

**验收标准**

- JSON、代码和普通文本均可 Diff。
- JSON 格式差异经过标准化。
- 左右输入类型不同时仍可对比。
- 格式化失败仍能完成文本对比。

### T08：增加应用内文本 Diff UI

**交付内容**

- 左右文本展示、行号和差异高亮。
- 新增、删除、修改状态展示。
- 滚动同步。
- 复制、保存和外部工具入口。

**验收标准**

- 不安装外部工具也能查看差异。
- JSON、代码、普通文本显示正确。
- 大文本输入不会明显阻塞界面。
- 现有外部 diff 工具入口不回归。

### T09：补齐单元测试与集成测试

**交付内容**

- JSON/JSONL、内容识别、格式化和通用 Diff 测试。
- URI、时间、Base64、QR Code 现有功能补测。
- 将 `tests` crate 改造成有效集成测试入口。
- 移除对本机 IDE 和网络环境的强依赖。

**验收标准**

- 每个核心功能具备正常、错误和边界测试。
- 测试可重复运行。
- `cargo test -p dev-kit` 稳定通过。
- CI 测试任务稳定通过。

### T10：建立 GitHub Release

**交付内容**

- 版本 tag 触发发布。
- 构建 macOS Apple Silicon、macOS Intel、Linux 和 Windows CLI。
- 构建 Tauri 桌面安装包。
- 上传 SHA256 校验文件。
- 自动生成 Release Notes。
- 发布前执行完整质量门禁。

**验收标准**

- 测试 tag 可生成完整 Release。
- CLI 和 GUI 资产均可下载。
- 每个资产都有校验值。
- 版本不一致或测试失败时发布停止。

### T11：增加 Homebrew Tap

**交付内容**

- 创建独立 Homebrew Tap。
- 增加 `devkit` Formula。
- 支持 macOS Intel 和 Apple Silicon。
- Release 后自动更新 Formula。
- 增加安装和升级文档。

**验收标准**

```bash
brew tap wenhaozhao/dev-kit
brew install devkit
devkit --version
```

- 安装成功。
- 版本与 GitHub Release 一致。
- Formula 中的 SHA256 与 Release 资产一致。

### T12：完成文档、官网和正式发布

**交付内容**

- 新增并维护 `CHANGELOG.md`。
- 重构 README。
- 创建官网并接入 GitHub Pages。
- 增加 CLI、GUI、Release 和 Homebrew 文档。
- 更新 Roadmap 和本版本文档。
- 执行 `v0.2.0` 正式发布检查。

**验收标准**

- 新用户可根据 README 完成安装和首次使用。
- 官网下载链接指向正确 Release。
- README、Changelog、Roadmap 和 Release 内容一致。
- `v0.2.0` 发布流程完整跑通。

## 5. 依赖关系

```text
T01
 ├── T02 ──┬── T03 ── T04
 │         └── T05 ── T06 ── T07 ── T08
 │                         └── T09
 └────────────────────────────── T10 ── T11
                                      └── T12
```

可并行执行的工作：

- T03 与 T05 在 T02 完成后可并行。
- T04 在 T03 完成后开始。
- T09 可在 T03、T06、T07 分别完成后分批补充。
- T10 的 CI 发布准备可在 T09 完成前进行，但正式发布门禁必须等待 T09。

## 6. 监督检查点

### Checkpoint A：工程基线

**范围：** T01–T02

必须确认：

- CI 可触发并通过。
- CLI 和 Tauri 构建顺序稳定。
- 版本一致性检查可用。

未通过时：暂停功能开发，先修复构建和质量门禁。

### Checkpoint B：JSONL 可演示

**范围：** T03–T04

必须确认：

- JSONL 能转换为 JSON Array。
- GUI 可格式化、查询、复制和保存。
- 非法行错误包含行号。

### Checkpoint C：通用文本 Diff 可演示

**范围：** T05–T08

必须确认：

- 目标格式可识别。
- 格式化失败可回退普通文本。
- 应用内 Diff 可独立使用。
- 外部 diff 工具仍可选。

### Checkpoint D：质量门禁

**范围：** T09

必须确认：

- 核心测试稳定通过。
- 无必须依赖本机 IDE 或网络的测试。
- Clippy、fmt 和 workspace 检查通过。

### Checkpoint E：测试发布链路

**范围：** T10–T11

必须确认：

- 测试 tag 可生成 Release。
- CLI、GUI 资产可下载和校验。
- Homebrew Tap 可安装正确版本。

### Checkpoint F：正式发布

**范围：** T12

必须确认：

- 文档、官网和 Release 链接一致。
- `CHANGELOG.md` 有 `v0.2.0` 条目。
- 所有发布门禁通过。
- 发布后执行安装和核心功能冒烟测试。

## 7. 发布门禁清单

正式创建 `v0.2.0` tag 前，必须全部满足：

- [ ] 工作区无未预期修改。
- [ ] `cargo fmt --all -- --check` 通过。
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- [ ] `cargo test -p dev-kit` 通过。
- [ ] workspace 构建通过。
- [ ] 前端构建通过。
- [ ] Tauri 构建通过。
- [ ] JSONL 功能冒烟测试通过。
- [ ] 通用文本 Diff 功能冒烟测试通过。
- [ ] README、CHANGELOG、ROADMAP 已更新。
- [ ] CLI、GUI、Homebrew 安装路径已验证。
- [ ] Release 资产和 SHA256 已验证。

## 8. 风险与应对

| 风险 | 影响 | 应对 |
|---|---|---|
| 代码格式化器跨平台行为不一致 | Diff 结果不稳定 | 内置格式化优先，外部 formatter 仅作为增强 |
| 大文本导致 UI 卡顿 | 用户体验下降 | 限制同步处理规模，必要时异步计算并显示状态 |
| JSON 与 JSONL 边界歧义 | 误识别 | 先整体 JSON 解析，失败后再尝试 JSONL |
| Tauri 构建依赖 deployment binary | Release 失败 | CI 显式先构建 CLI，再执行桌面构建 |
| Homebrew Formula SHA256 不一致 | 无法安装 | Release 后自动生成并校验 Formula |
| 发布版本信息不一致 | 用户安装错误版本 | tag 发布前执行统一版本检查 |

## 9. 回滚策略

- 新格式化和通用 Diff 通过功能开关或保留旧 JSON 路径进行回退。
- JSONL 解析失败时不得破坏标准 JSON 解析路径。
- Release 流程失败时不创建正式 tag，使用测试 tag 重试。
- Homebrew Formula 发布错误时回退到上一版 Formula。
- 官网下载链接始终保留上一稳定版本入口。

## 10. 完成定义

当以下条件全部满足时，`v0.2.0` 才算完成：

1. T01–T12 全部为 `Done`。
2. 所有 Checkpoint 均通过。
3. 发布门禁清单全部勾选。
4. GitHub Release 已发布且资产可下载。
5. Homebrew Tap 可安装并验证版本。
6. 官网、README、Changelog 和 Roadmap 已同步。
7. 发布后冒烟测试未发现阻断性问题。

## 11. 进度记录

| 日期 | 检查点/任务 | 结果 | 备注 |
|---|---|---|---|
| 2026-07-17 | 版本开发文档建立 | Done | 创建 v0.2.0 执行基线 |
| 2026-07-17 | T01：建立版本发布基线 | Done | `./scripts/check-version.sh` 与 `cargo check -p dev-kit` 通过 |
| 2026-07-17 | Checkpoint A / T02 | Done | fmt、Clippy、测试、workspace 与前端构建通过；Tauri sidecar 契约已验证 |
| 2026-07-17 | T03：JSON/JSONL 核心解析 | Done | JSONL 数组转换、空行跳过和行号错误测试通过 |
| 2026-07-17 | T04：Json Parser 接入 JSONL | Done | 输入识别状态、错误提示与现有操作路径已验证 |
| 2026-07-17 | T05：内容类型识别 | Done | 12 种目标类型、普通文本回退与覆盖逻辑测试通过 |
| 2026-07-17 | T06：统一文本格式化 | Done | 结构化数据与普通文本格式化测试通过 |
| 2026-07-17 | T07：通用文本 Diff | Done | 自动识别、格式化回退、混合类型与行级 Diff 测试通过 |
| 2026-07-17 | T08：应用内文本 Diff UI | Done | 行级状态、双侧行号、滚动同步、复制保存与外部入口已验证 |
| 2026-07-17 | T09：单元测试与集成测试 | Done | workspace 17 个单测与 4 个离线集成测试通过 |
| 2026-07-17 | T11：Homebrew Tap 准备 | In Progress | Formula 模板、渲染脚本与更新工作流已验证；待外部 Tap 配置 |
