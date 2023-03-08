# shadower
_a simple command-line utility to add rounded corners and shadows to images_

## Building:
1. make sure you have rust installed
2. `cargo build --release`

## Usage
shadower always reads the image from stdin and outputs it to stdout, so here are some examples:

- `wl-paste | shadower | wl-copy`
- `shadower < image.png > image_shadow.png`
- `grimblast save active - | shadower | wl-copy`

## Contributing

1. Make a fork
2. Make your changes
3. Create a PR

No need to ask me if you can implement something, if I think it's a good thing I'll merge it in.
