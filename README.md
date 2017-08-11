# github-deployment

Send deployment events to the Github

# Release

```
$ make release
$ ls -l releases/x86_64-unknown-linux-gnu/
```

# Development

1. Install [rustup.rs](https://www.rustup.rs)

    ```bash
    $ curl https://sh.rustup.rs -sSf | sh
    ```

2. Install latest [rust-lang](http://rust-lang.org) (let's say `1.19.0`)

    ```bash
    $ rustup update
    $ rustup default 1.19.0
    ```

# TODO

- [x] Better `compilation.mk` with mounting
- [ ] Better debug output and error messages
