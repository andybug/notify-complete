# Notify Complete

`notify-complete` is a simple utility to that runs a command and then displays a
system notification upon completion. Various aspects of the notification can be
configured via the CLI or a config file.

## Installation

### Arch Linux (AUR)

```shell
paru -S notify-complete
```

### Cargo

```shell
git clone https://github.com/andybug/notify-complete.git
cd notify-complete
cargo install --path .
```

## Usage

```
notify-complete [OPTIONS] <command>
```

| Option            | Effect                                                                                       |
| ----------------- | -------------------------------------------------------------------------------------------- |
| `-t`, `--title`   | The title (headline) of the notification                                                     |
| `-m`, `--message` | The message (body) of the notification                                                       |
| `-o`, `--timeout` | How long the notification will remain before hiding. See [below](#timeout) for more details. |
| `-u`, `--urgency` | Set the "priority" of the notification. See [below](#urgency) for more details.              |
| `-p`, `--profile` | Which profile to use from the [config](#configuration).                                      |

### Timeout

The timeout of the notification in milliseconds, or one of the following:

- `default` uses the system-wide notification timeout setting.
- `never` keeps the notification displayed until dismissed.

### Urgency

Some systems may treat notifications differently based on their urgency. The
following values are permitted:

- `low`
- `normal`
- `critical`

### Examples

The following will use the default settings. After five seconds a notification
will be displayed.

```shell
notify-complete sleep 5
```

In this example, after the `aws` command completes, a notification will be
displayed saying _Upload complete_ and will remain visible until the user
dismisses it. Note that the `--` is optional, but it does make a clear delimiter
between the `notify-complete` options and the command.

```shell
notify-complete --title "Upload complete" --timeout never -- aws s3 sync . s3://bucket
```

Here, we use a profile defined in the configuration file, but overwrite the
urgency.

```shell
notify-complete --profile some-profile --urgency low -- ./my-script.sh
```

## Configuration

The configuration file is located at `~/.config/notify-complete/config.toml` on
Linux and Mac systems.

Configurations are grouped in _profiles_. Multiple profiles can be specified in
the config and the defaults can even be overwritten. When running
`notify-complete`, the profile can be specified with the `--profile` flag. An
example configuration is presented below.

```toml
[[profile]]
name = "alert" # this is the only required field
title = "Something bad happened"
urgency = "critical"
timeout = "never"

[[profile]]
name = "download"
title = "Download complete"
message = "Time to work"
timeout = "30000" # note that timeout must be a string. This represents 30 seconds

[[profile]]
name = "default" # overwrite the default settings; you do not need to specify --profile default
title = "notify-complete"
message = "Command has finished"
```
