# DevKit v0.2.1 开发计划

> 版本主题：Homebrew 安装验收与发布链路加固
>
> 前置条件：`v0.2.0` 主干功能、Release 工作流和文档已完成并发布。

## 目标

- 创建并配置 `wenhaozhao/homebrew-dev-kit` Tap。
- 以 `v0.2.1` Release 资产生成并更新 `devkit` Formula。
- 在 macOS Apple Silicon 与 Intel 环境验证安装、升级和版本一致性。
- 记录安装失败时的回滚、修复和重试流程。

## 任务

| ID | 任务 | 状态 | 验收 |
|---|---|---|---|
| H01 | 创建 Tap 并配置 Actions 写入凭据 | Planned | Release 工作流可更新 Formula |
| H02 | 发布 v0.2.1 测试资产并生成校验值 | Planned | Formula SHA256 与资产一致 |
| H03 | 验证安装与升级 | Planned | `brew tap`、`brew install`、`devkit --version` 通过 |
| H04 | 验证 Intel/Apple Silicon 与回滚流程 | Planned | 两种架构可安装，失败可恢复上一 Formula |

## 验收命令

```bash
brew tap wenhaozhao/dev-kit
brew install devkit
devkit --version
brew upgrade devkit
```

## 不包含

- 不修改当前 `v0.2.0` 的版本号或 Release tag。
- 不以 Homebrew 验收阻塞 `v0.2.0` 主干功能开发。
