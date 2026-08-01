a fork/clone of https://github.com/yozhgoor/CreateProcessW  

Changes:  
- MIT license.
- rename crate.
- adding tests.
- splitting to multiple files.
- exposing `SW_*` flags, and supporting initializing `child` process with it.
- adding verbose rust documentation.

<hr/>

remained TODO:  
- adding examples and more rust documentation, as well as more tests.  

- support native Windows API for signals, forwarding.  
- create a child process as process-group-root, which is needed to send a signal to it.  

- improving exit code handling for signals.  
- support for single and multiple args, in addition to the command-line.  

- timing, a run of child process, until end.
- time-limit for child-process run.  

- `tee`. mirror `STDOUT` and `STDERR` to file(s).

- privilege run, admin, system NT, implicit/explicit with credentials (plain text).

<hr/>

build + documentation + open documentation:  

```rust
cargo build
cargo doc
cargo doc --open
```