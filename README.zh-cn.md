# Dev Kit

A collection of utilities and tools for development.

[English](./README.md)

## Features

### 1. URI Tools
- **Decode**: 解码 URI 组件。
  ```shell
  $ devkit uri decode 'foo%20bar'
  foo bar
  ```

### 2. JSON Tools
所有 JSON 工具均支持以下输入类型：
- **JSON String**: 直接输入 JSON 字符串。
  ```shell
  $ devkit json beauty '{"a":1}'
  ```
- **File**: JSON 文件路径。
  ```shell
  $ devkit json beauty data.json
  ```
- **URL**: JSON 资源链接。
  ```shell
  $ devkit json beauty https://api.example.com/data.json
  ```
- **Command**: 输出 JSON 的 Shell 命令。
  ```shell
  $ devkit json beauty 'cat data.json'
  ```
- **Unix Pipe**: 通过 `-` 接收标准输入（stdin）。
  ```shell
  $ cat data.json | devkit json beauty -
  ```
- **JetBrains HTTP**: 支持 JetBrains HTTP Client 请求语法。
  ```shell
  $ devkit json beauty 'GET https://api.example.com/data.json
  Accept: application/json'
  ```

#### 命令：
- **Beauty**: 格式化 JSON 字符串或文件。支持通过别名 `format` 使用。
  ```shell
  $ devkit json beauty '{"a":1,"b":2}'
  {
    "a": 1,
    "b": 2
  }
  ```
- **Query**: 使用 JSONPath 提取内容。支持通过别名 `search` 使用。
  ```shell
  $ devkit json query -q '$.a' '{"a":1,"b":2}'
  1
  ```
- **Diff**: 对比两个 JSON 对象，支持 JSONPath 提取后对比，并可调用外部 Diff 工具（如 IDEA, VSCode, Zed）。
  ```shell
  $ devkit json diff '{"a":1}' '{"a":2}' --diff-tool vscode
  ```

### 3. Time Tools
- **Now**: 获取当前时间，支持指定时区和格式。
  ```shell
  # 获取当前时间（RFC3339）
  $ devkit time now
  2023-10-27T10:00:00+08:00

  # 获取当前毫秒时间戳
  $ devkit time now -f ts
  1698372000000
  ```
- **Parse**: 解析时间字符串或时间戳。
  ```shell
  # 解析时间戳
  $ devkit time parse 1698372000000
  2023-10-27T10:00:00+08:00

  # 解析字符串并转换格式
  $ devkit time parse "2023-10-27 10:00:00" -f ts
  1698372000000
  ```

## 安装

```shell
cargo install --path ./dk
```

### 环境变量

为了在任何地方都能使用 `devkit` 命令，请确保 Cargo 的二进制目录已添加到你的 `PATH` 中。

对于大多数用户，这意味着将 `~/.cargo/bin` 添加到你的 `PATH` 环境变量中。

#### macOS/Linux

在你的 `.bashrc`、`.zshrc` 或等效文件中添加以下行：

```shell
export PATH="$HOME/.cargo/bin:$PATH"
```

#### Windows

通过系统环境变量设置将 `%USERPROFILE%\.cargo\bin` 添加到你的 `Path` 环境变量中。

## 使用

```shell
devkit --help
```

