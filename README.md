# Grid Tailstream

Grid Tailstream is a file watching and log streaming tool that monitors files for appended data and forwards new content to configurable endpoints in real time. It functions like `tail -f` but with pluggable output backends, making it suitable for forwarding logs and metrics from local files to remote aggregation services.

## What this is

Grid Tailstream watches a specified file and streams any newly appended content to a configured output destination. By default it prints to the console (acting like `tail -f`), but it can also publish to Redis Pub/Sub channels or send binary messages over WebSockets. Output chunks can optionally be compressed with gzip.

## What this repository contains

- **`tailstream` binary** — A Rust CLI tool for watching files and streaming output
- **Console output backend** — Prints streamed content to stdout
- **Redis Pub/Sub backend** — Publishes log chunks to a specified Redis channel
- **WebSocket backend** — Sends log chunks as binary messages to a WebSocket server
- **Configurable compression** — gzip or none, applied per chunk

## Role in the stack

Grid Tailstream operates as a log collection and forwarding component within the broader infrastructure stack. It can run on nodes and services to stream local log files to centralized aggregation systems. It fits into the observability layer alongside metrics and logging infrastructure.

## Relation to ThreeFold

This technology is used within the ThreeFold ecosystem and was first deployed on the ThreeFold Grid. The component itself is designed as reusable infrastructure technology and should be understood by its technical function first, independent of any specific deployment.

## Ownership

This repository is owned and maintained by TF-Tech NV, a Belgian company responsible for the development and maintenance of this technology.

## Output

Output is configured via the `--output` flag, which accepts a URL.

Currently Grid Tailstream supports three output types:

- `console://` [default] — Prints the file content to console. Does not accept any extra arguments.
- `redis://[<username>][:<password>@]<hostname>[:port]/<channel>` — All log chunks are `PUBLISH`ed to the specified `channel`. The channel can be any valid Redis Pub/Sub channel name.
- `ws[s]://<server>[:port]/[path]` — All log chunks are sent to the WebSocket as binary messages. The server can then decide what to do with the messages.

## Compression

By default chunks are compressed with gzip. Currently supported compression algorithms:

- `none` — No compression; chunks are pushed as-is.
- `gzip` [default] — gzip compression.

> When compression is used, the receiver must decompress chunks before writing them to a log file.

## Usage

```
tailstream
A tail-like tool that streams file output to a configurable destination.

USAGE:
    tailstream [OPTIONS] <FILE>

ARGS:
    <FILE>    File to watch

OPTIONS:
    -c, --compression <COMPRESSION>    Compression algorithm applied per chunk.
                                       Ignored for console output. [default: gzip]
    -d, --debug                        Enable debug logs
    -h, --help                         Print help information
    -o, --output <OUTPUT>              Output stream. Defaults to console://.
                                       Supports redis:// and ws:// / wss://.
                                       [default: console://]
    -t, --tail <TAIL>                  Output the last N bytes before streaming.
                                       [default: 8192]
    -V, --version                      Print version information
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
Copyright (c) TFTech NV.
