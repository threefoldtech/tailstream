## tailstream
`tailstream` is like `tail -f` but streams the output based on command line config. By default file is tailed and streamed to console (act like tail)
but you can configure the output to send the data differently

### Output
Output is configured via the `--output` which accepts a url.

Currently tailstream supports three output types:
- `console://` [default] this output prints the file content to console. It doesn't accept any extra arguments
- `redis://[<username>][:<password>@]<hostname>[:port][/<db>]/<channel>`. All logs (chunks) are `PUBLISH`ED to the `channel`. `channel` can be any valid PUB/SUB redis channel name.
- `ws[s]://<server>[:port]/[path]`. All logs (chunks) are sent to the websocket as `binary` messages. The server then can decide what to do with the messages.

### Compression
By default chunks are compressed with `gzip` compression algorithm. Currently the only supported compressions algorithms are:

- `none`: no compression, chunks are pushed as is
- `gzip`: [default] gzip compression.

> When compression is used, the receiver of the log chunks need to un-compress them before writing them again into a log file.


## Usage
```
tailstream
A tail like tool but tail the file to a configurable output module

USAGE:
    tailstream [OPTIONS] <FILE>

ARGS:
    <FILE>    file to watch

OPTIONS:
    -c, --compression <COMPRESSION>    compression, compresses the log message (per chunk) so each
                                       chunk of logs can be decompressed separately from previous
                                       chunks ignored in case of `console` output [default: gzip]
    -d, --debug                        enable debug logs
    -h, --help                         Print help information
    -o, --output <OUTPUT>              Output stream. defaults to console://. Other output supports
                                       redis://[user:password@]<address[:port]>
                                       ws[s]://address[:port]/[prefix] [default: console://]
    -t, --tail <TAIL>                  output the last TAIL bytes default to 8k [default: 8192]
    -V, --version                      Print version information
```
