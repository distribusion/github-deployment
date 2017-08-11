# github-deployment

Send deployment events to the Github

```
USAGE:
    github-deployment [FLAGS] [OPTIONS] <repo> --head <head> --status <status>

FLAGS:
    -d, --debug      Show the debug messages
    -h, --help       Prints help information
    -q, --quiet      Exit without a failure even if shit happened
    -V, --version    Prints version information

OPTIONS:
        --base <base>        The ref of the previous deployment. This can be a branch, tag, or SHA
        --head <head>        The ref of the current deployment. This can be a branch, tag, or SHA
        --status <status>    A deployment status to be set [default: pending]  [values: pending, error, success, failure]

ARGS:
    <repo>    A Github repository path as <owner>/<repo>
```

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

3. Build the binary and for simplicity link it

    ```bash
    $ make
    $ ln -s ./target/debug/github-deployment ./github-deployment
    ```


# TODO

- [x] Better `compilation.mk` with mounting
- [ ] Better debug output and error messages
