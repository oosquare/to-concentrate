## To Concentrate

### Overview

`to-concentrate` is a notifier daemon written in Rust which makes practical use of the tomato clock method.

### Installation

#### Nix

The installation requires that Nix Flake is enabled. Run the command below to install programs to your Nix profile:

```bash
nix profile install github:oo-infty/to-concentrate#
```

If you're using NixOS, you can alternatively use the following `flake.nix`:

```nix
{
  description = "Example flake.nix with `to-concentrate` installed";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    to-concentrate = {
      url = "github:oo-infty/to-concentrate";
      inputs.nixpkgs.follows = nixpkgs;
    };
  };

  output = { self, ... }@inputs: {
    nixosConfigurations.example-machine = inputs.nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";

      modules = [
        # Put your other NixOS modules here.

        ({ pkgs, ... }: {
          environment.systemPackages = [
            inputs.to-concentrate.packages.${pkgs.system}.to-concentrate
          ];
        })
      ];
    };
  };
}
```

#### Source

```bash
git clone https://github.com/oo-infty/to-concentrate.git
cd to-concentrate
cargo install --path .
```

### Usage

To Concentrate includes two executables:

- `to-concentrate-daemon`: a daemon which is responsible for notification.
- `to-concentrate`: a client which controls the daemon.

The daemon's usage:

```plain
Usage: to-concentrate-daemon [OPTIONS]

Options:
  -c, --config <CONFIG>        Path to a custom configuration file
  -v, --verbosity <VERBOSITY>  Maximum logging level the subscriber should use [default: INFO]
  -d, --daemonize              Whether to daemonize the process
  -h, --help                   Print help
  -V, --version                Print version
```

The client's usage:

```plain
Usage: to-concentrate [OPTIONS] <COMMAND>

Commands:
  init    Launch and initialize a daemon process
  pause   Pause the timer
  resume  Resume the timer
  query   Query the timer's status. Show all information if no flag is specified
  skip    Skip the current stage
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Path to a custom configuration file
  -h, --help             Print help
  -V, --version          Print version
```

For more details, run `to-concentrate help <COMMAND>`.

### Configuration

By default, both daemon and client will read your configuration file in `$XDG_CONFIG_HOME/to-concentrate/config.toml` (usually ``$HOME/.config/to-concentrate/config.toml``). If you haven't place your configuration there yet, the program will automatically generate one.

An example configuration file is presented below:

```toml
# This configuration file is generated automatically. Feel free to do some
# modification.

# The `duration` section specifies the duration of each stage in seconds.
[duration]
preparation = 900
concentration = 2400
relaxation = 600

# The `notification.<stage>` section specifies the message shown in desktop
# notifications. `body` is optional.
[notification.preparation]
summary = "Preparation Stage End"
body = "It's time to start concentrating on learning."

[notification.concentration]
summary = "Concentration Stage End"
body = "Well done! Remember to have a rest."

[notification.relaxation]
summary = "Relaxation Stage End"
body = "Feel energetic now? Let's continue."

# The `runtime` section specifies the paths to some runtime files. Leave
# them empty to use default settings. Currently environment variables is not
# supported.
# [runtime]
# socket = "/path/to/unix/socket"
# runtime = "/path/to/pid/file"
```
