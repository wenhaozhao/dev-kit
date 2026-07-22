# DevKit v0.2.1 开发计划

> 版本主题：内核驱动的统一内容解析与瘦前端交互
>
> 当前原则：输入类型由 DevKit Rust 内核解析结果决定，UI 不自行执行 JSON/JSONL 检测。

## 目标

- 统一 JSON、JSONL、TOML 与纯文本的内核表示和格式化输出。
- 让 JSON Parser 与 Content Diff 使用内核返回的 `input_type`，避免 Fat UI 在大内容上重复解析。
- 保持输入框允许任意可导出 JSON 的来源表达式，包括文件路径、URL、curl 和 JetBrains HTTP 请求。
- 以可接受的解析优先级处理内容：先尝试完整 JSON，再尝试 JSONL，最后回退到 TOML/纯文本。

## 任务

| ID | 任务 | 状态 | 验收 |
|---|---|---|---|
| K01 | 引入 `FormattedValue`，统一 JSON/JSONL/TOML/文本表示 | Done | Rust workspace 测试通过 |
| K02 | JSON Parser 返回内核解析类型并由 UI 展示 | Done | `jsonparser_query_json` 返回 `data` 与 `input_type` |
| K03 | Content Diff 接入统一格式解析与查询 | Done | JSON/TOML/文本输入可由内核处理 |
| K04 | JSONL 解析优先级与兼容性验证 | Done | 先 JSON、后 JSONL，失败后回退文本 |
| K05 | 内联文本对比 | Deferred | v0.2.1 暂不支持，使用外部 diff 工具 |
| H01 | Homebrew Tap、Formula、安装升级验收 | Deferred | 转入后续发布/安装专项计划 |

## 已确认的设计约束

- 不在 Vue/JavaScript 层调用 `JSON.parse` 或通过换行猜测输入类型。
- JSONL 的识别属于内核内容解析流程；当前实现先尝试完整 JSON，再尝试 JSONL。
- 内联对比功能暂不支持，不作为 v0.2.1 的阻塞项。
- 发布前统一更新 Cargo、npm 与 Tauri 的版本号为 `0.2.1`。

## 验证命令

```bash
cargo test --workspace
(cd dkui && npm run build)
```
