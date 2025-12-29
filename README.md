# Dev Kit

A collection of utilities and tools for development.

[中文版](./README.zh-cn.md)

## Features

### 1. URI Tools
- **Decode**: Decode URI components.
  ```shell
  $ dev-kit uri decode 'foo%20bar'
  foo bar
  ```

### 2. JSON Tools
All JSON tools support the following input types:
- **JSON String**: Direct JSON string.
  ```shell
  $ dev-kit json beauty '{"a":1}'
  ```
- **File**: Path to a JSON file.
  ```shell
  $ dev-kit json beauty data.json
  ```
- **URL**: URL to a JSON resource.
  ```shell
  $ dev-kit json beauty https://api.example.com/data.json
  ```
- **Command**: Shell command that outputs JSON.
  ```shell
  $ dev-kit json beauty 'cat data.json'
  ```
- **Unix Pipe**: Input from stdin using `-`.
  ```shell
  $ cat data.json | dev-kit json beauty -
  ```

#### Commands:
- **Beauty**: Format JSON strings or files. Supports alias `format`.
  ```shell
  $ dev-kit json beauty '{"a":1,"b":2}'
  {
    "a": 1,
    "b": 2
  }
  ```
- **Query**: Extract content using JSONPath. Supports alias `search`.
  ```shell
  $ dev-kit json query -q '$.a' '{"a":1,"b":2}'
  1
  ```
- **Diff**: Compare two JSON objects, supports comparison after JSONPath extraction, and can call external Diff tools (e.g., IDEA, VSCode, Zed).
  ```shell
  $ dev-kit json diff '{"a":1}' '{"a":2}' --diff-tool vscode
  ```

### 3. Time Tools
- **Now**: Get the current time, supports specified timezones and formats.
  ```shell
  # Get current time (RFC3339)
  $ dev-kit time now
  2023-10-27T10:00:00+08:00

  # Get current millisecond timestamp
  $ dev-kit time now -f ts
  1698372000000
  ```
- **Parse**: Parse time strings or timestamps.
  ```shell
  # Parse timestamp
  $ dev-kit time parse 1698372000000
  2023-10-27T10:00:00+08:00

  # Parse string and convert format
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

