# Dev Kit

<img src="./dkui/public/dk_icon.png" style="width: 128px;">

A collection of utilities and tools for development.

## Features

### 1. URI Tools
- **Decode**: Decode URI components. Supports alias `d`.
  ```shell
  $ devkit uri decode 'foo%20bar'
  foo bar
  ```
- **Encode**: Encode URI components. Supports alias `e`.
  ```shell
  $ devkit uri encode 'foo bar'
  foo%20bar
  ```
- **Parse**: Parse URI and extract components. Supports alias `p`.
  ```shell
  $ devkit uri parse 'https://example.com/path?a=1&b=2'
  scheme: https
  host: example.com
  port: 443
  path: /path
  query:
     a=1
     b=2
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
- **Options**:
    - `-q, --query <JSONPATH>`: Extract content using JSONPath before processing.
    - `-f, --file <FILE>`: Write output to a file (for Beauty and Query).
    - `--diff-tool <TOOL>`: Diff tool to use: `idea`, `vscode`, `zed`, etc.

### 3. Time Tools
- **Now**: Get the current time, supports specified timezones and formats.
  ```shell
  # Get current time (RFC3339)
  $ devkit time now
  2023-10-27T10:00:00+08:00

  # Get current millisecond timestamp
  $ devkit time now -f ts
  1698372000000

  # Get current time with custom format
  $ devkit time now -f "%Y-%m-%d %H:%M:%S"
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
- **Options**:
    - `-z, --tz <TIMEZONE>`: Specify timezone (e.g., `+08:00`).
    - `-f, --format <FORMAT>`: Output format: `rfc3339`, `ts`, or custom format (e.g., `%Y-%m-%d`).
    - `--iu, --input-unit <UNIT>`: Input timestamp unit: `s`, `ms`.
    - `--ou, --output-unit <UNIT>`: Output timestamp unit: `s`, `ms`.

### 4. Base64 Tools
- **Encode**: Encode string to Base64. Supports alias `e`.
  ```shell
  $ devkit base64 encode 'hello world'
  aGVsbG8gd29ybGQ=
  ```
- **Decode**: Decode Base64 string. Supports alias `d`.
  ```shell
  $ devkit base64 decode 'aGVsbG8gd29ybGQ='
  hello world
  ```
- **Options**:
    - `-u, --url-safe`: Use URL-safe Base64.
    - `-n, --no-pad`: No padding.
    - `-f, --file <FILE>`: Write output to a file.

### 5. QR Code Tools
Generate QR codes from text or URLs. Supports alias `qrcode`, `qr`.
- **Generate**:
  ```shell
  # Output as text (default)
  $ devkit qrcode 'https://github.com'

  # Save as image
  $ devkit qrcode 'https://github.com' -t image -f qr.png
  ```
- **Options**:
    - `-t, --type <TYPE>`: Output type: `text` (default), `image`, `svg`.
    - `-e, --ec-level <LEVEL>`: Error correction level: `l` (7%), `m` (15%), `q` (25%), `h` (30%).
    - `-v, --version <VERSION>`: QR code version (1-40).
    - `-p, --plain`: Plain text output without details.

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

