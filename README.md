<div align="center" width="100%">

# shadower

_a simple command-line utility to add rounded corners and shadows to images_

![GitHub](https://img.shields.io/github/license/n3oney/shadower)
![GitHub top language](https://img.shields.io/github/languages/top/n3oney/shadower?color=%23DEA584&logo=rust)
[![Crates.io](https://img.shields.io/crates/d/shadower?label=cargo%20downloads)](https://crates.io/crates/shadower)
[![AUR votes](https://img.shields.io/aur/votes/shadower-git?label=AUR%20votes)](https://aur.archlinux.org/packages/shadower-git)
[![wakatime](https://wakatime.com/badge/github/n3oney/shadower.svg)](https://wakatime.com/badge/github/n3oney/shadower)

</div>

## Installation

### From AUR

`paru -S shadower-git`

### From source

`cargo build --release`

### From crates.io

`cargo install shadower`

## Usage

shadower by default reads the image from stdin and outputs it to stdout, so here are some non-config examples:

- `wl-paste | shadower | wl-copy`
- `shadower < image.png > image_shadow.png`
- `grimblast save active - | shadower | wl-copy`

However, you can also customize every aspect of the rounding, shadows, and padding of the images.
To do so, you can use **math expressions**, which let you dynamically calculate the values depending on the size of the input image.
For example, setting the `--offset-y` to `max / 6 / 4`, while running on a 800x600px image, will result in the shadow being offset down by 33.(3)px.
To view all the flags, use `--help`.

## Config file

You can also configure shadower using a config file instead of the flags. The default config location is `$XDG_CONFIG_HOME/shadower/config.toml`, but you can override it using the `--config` flag.
The config file can change every option (except the `--config`), and you just have to put the values in.
The keys have to use snake_case instead of kebab-case.

**Example config:**

```toml
radius="20"
padding_x="15 + max/2"
```

You can use environment variables, they will be expanded.

The default config values are always displayed in the `--help`.

###### Note: flags have priority over the config file

## Contributing

1. Make a fork
2. Make your changes
3. Commit the changes (please use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)!)
4. Create a PR

No need to ask me if you can implement something, if I think it's a good thing I'll merge it in.
