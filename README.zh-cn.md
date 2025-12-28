# Dev Kit

A collection of utilities and tools for development.

[English](./README.md)

## Features

### 1. URI Tools
- **Decode**: 解码 URI 组件。
  ```shell
  $ dev-kit uri decode 'foo%20bar'
  foo bar
  ```

### 2. JSON Tools
- **Beauty**: 格式化 JSON 字符串或文件。支持通过别名 `format` 使用。
  ```shell
  $ dev-kit json beauty '{"a":1,"b":2}'
  {
    "a": 1,
    "b": 2
  }
  ```
- **Query**: 使用 JSONPath 提取内容。支持通过别名 `search` 使用。
  ```shell
  $ dev-kit json query -q '$.a' '{"a":1,"b":2}'
  1
  ```
- **Diff**: 对比两个 JSON 对象，支持 JSONPath 提取后对比，并可调用外部 Diff 工具（如 IDEA, VSCode, Zed）。
  ```shell
  $ dev-kit json diff '{"a":1}' '{"a":2}' --diff-tool vscode
  ```

### 3. Time Tools
- **Now**: 获取当前时间，支持指定时区和格式。
  ```shell
  # 获取当前时间（RFC3339）
  $ dev-kit time now
  2023-10-27T10:00:00+08:00

  # 获取当前毫秒时间戳
  $ dev-kit time now -f ts
  1698372000000
  ```
- **Parse**: 解析时间字符串或时间戳。
  ```shell
  # 解析时间戳
  $ dev-kit time parse 1698372000000
  2023-10-27T10:00:00+08:00

  # 解析字符串并转换格式
  $ dev-kit time parse "2023-10-27 10:00:00" -f ts
  1698372000000
  ```

## Installation

```shell
cargo install --path .
```

## Usage

```shell
dev-kit --help
```

