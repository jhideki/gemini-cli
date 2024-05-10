# gemini-cli

A simple command line wrapper for the google gemini ai model.

## Requirements

- Rust
- [Gemini api key](https://ai.google.dev/gemini-api/docs/api-key)

## Features

- Full message thread support
- Write code snippets to files

## Installation

1. Install rust
2. Clone this repo

```bash
git clone https://github.com/jhideki/gemini-cli.git
```

3. Add your gemini api key as an environment variable
```bash
GEMINI_API_KEY=<your api key>
```

4. Build project
```bash
carbo build --release
```
5. Add executable to system PATH
```bash
export PATH="/home/user/gemini-cli/target/release:$PATH"
```

## Usage
To start a message call
```bash
gemini-cli
```

The cli will prompt you type in a query. You can pass gemini files by specifiying the file path '<>'
E.g.,
```bash
Enter a prompt:
Explain what this file does <main.rs>
```

<br>

If the response includes a code snippet, gemini-cli will the write the code to a file within the './responses' directory with the relevant extension.

