# rebazel [![Build Status](https://travis-ci.org/meetup/rebazel.svg?branch=master)](https://travis-ci.org/meetup/rebazel)

> a tool for expediting [bazel build](https://bazel.build/) workflows

## Installation

`rebazel` can be installed as a standalone binary Darwin and Linux operating systems.

### Github releases

You can download a released binary directly from [Github releases](https://github.com/meetup/rebazel/releases).

You can also download a release directly with curl.

```bash
$ cd $HOME/bin
$ curl -L "https://github.com/meetup/rebazel/releases/download/v0.1.0/rebazel-$(uname -s)-$(uname -m).tar.gz" \
  | tar -xz
```

Ensure `$HOME/bin` is on your `$PATH` variable and you should be good to go.

### Homebrew

If you are using OSX, it's likely you're using [homebrew](https://brew.sh/) to manage your packages. You can install
rebazel using homebrew with the following command.

```bash
$ brew install meetup/tools/rebazel
```

## Usage

Just type `rebazel` where you would normally type `bazel`. That's it.

`rebazel` will watch the provided target's source and build files for changes and retrigger the action where appropriate.

```bash
$ rebazel test --test_filter=com.foo.api.* --test_output=streamed //foo:test
```

Will run the tests for `//foo:test` target and watch all of its associated sources and build dependencies.

By default, forwards the command line to the `bazel` executable on the users `PATH`. If you which to use an alternate
executable export the `REBAZEL_BAZEL_EXEC` env variable set to the path of your bazel executable.

By default, `rebazel` will debounce actions so that they happen no more frequently than 100 milliseconds. This is also configurable by
exporting the env variable `REBAZEL_DEBOUNCE_DELAY`.

`rebazel` uses a configurable level of logging though the env variable `RUST_LOG`, specified by then [env_log](https://doc.rust-lang.org/log/env_logger/#enabling-logging) crate. By default its set to `info` but you may which to set it to `debug`
to see exactly which files will be watched for a given run.

Meetup 2017
