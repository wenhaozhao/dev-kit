# Dev Kit

A collection of utilities and tools for development.

[中文版](./README.zh-cn.md)

## Features

### 1. URI Tools
- **Decode**: Decode URI components.
  ```shell
  $ devkit uri decode 'foo%20bar'
  foo bar
  ```

### 2. JSON Tools
All JSON tools support the following input types:
- **JSON String**: Direct JSON string.
  ```shell
  $ devkit json beauty '{"a":1}'
  ```
- **File**: Path to a JSON file.
  ```shell
  $ devkit json beauty data.json
  ```
- **URL**: URL to a JSON resource.
  ```shell
  $ devkit json beauty https://api.example.com/data.json
  ```
- **Command**: Shell command that outputs JSON.
  ```shell
  $ devkit json beauty 'cat data.json'
  ```
- **Unix Pipe**: Input from stdin using `-`.
  ```shell
  $ cat data.json | devkit json beauty -
  ```
- **JetBrains HTTP**: JetBrains HTTP Client request syntax.
  ```shell
  $ devkit json beauty 'GET https://api.example.com/data.json
  Accept: application/json'
  ```

#### Commands:
- **Beauty**: Format JSON strings or files. Supports alias `format`.
  ```shell
  $ devkit json beauty '{"a":1,"b":2}'
  {
    "a": 1,
    "b": 2
  }
  ```
- **Query**: Extract content using JSONPath. Supports alias `search`.
  ```shell
  $ devkit json query -q '$.a' '{"a":1,"b":2}'
  1
  ```
- **Diff**: Compare two JSON objects, supports comparison after JSONPath extraction, and can call external Diff tools (e.g., IDEA, VSCode, Zed).
  ```shell
  $ devkit json diff '{"a":1}' '{"a":2}' --diff-tool vscode
  ```

### 3. Time Tools
- **Now**: Get the current time, supports specified timezones and formats.
  ```shell
  # Get current time (RFC3339)
  $ devkit time now
  2023-10-27T10:00:00+08:00

  # Get current millisecond timestamp
  $ devkit time now -f ts
  1698372000000
  ```
- **Parse**: Parse time strings or timestamps.
  ```shell
  # Parse timestamp
  $ devkit time parse 1698372000000
  2023-10-27T10:00:00+08:00

  # Parse string and convert format
  $ devkit time parse "2023-10-27 10:00:00" -f ts
  1698372000000
  ```

## Installation

```shell
cargo install --path ./dk
```

### Environment Variables

To use the `devkit` command from anywhere, ensure that the Cargo binary directory is in your `PATH`.

For most users, this means adding `~/.cargo/bin` to your `PATH` environment variable.

#### macOS/Linux

Add this line to your `.bashrc`, `.zshrc`, or equivalent:

```shell
export PATH="$HOME/.cargo/bin:$PATH"
```

#### Windows

Add `%USERPROFILE%\.cargo\bin` to your `Path` environment variable via the System Environment Variables settings.

## Usage

```shell
devkit --help
```

