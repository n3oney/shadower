# shadower

_a simple command-line utility to add rounded corners and shadows to images_

## Building:

1. make sure you have rust installed
2. `cargo build --release`

## Usage

shadower by default reads the image from stdin and outputs it to stdout, so here are some non-config examples:

- `wl-paste | shadower | wl-copy`
- `shadower < image.png > image_shadow.png`
- `grimblast save active - | shadower | wl-copy`

However, you can also customize every aspect of the rounding, shadows, and padding of the images.
To do so, you can use **math expressions**, which let you dynamically calculate the values depending on the size of the input image.
For example, setting the `--offset-y` to `max / 6 / 4`, while running on a 800x600px image, will result in the shadow being offset down by 33.(3)px.
To view all the flags, use `--help`.

## Contributing

1. Make a fork
2. Make your changes
3. Commit the changes (please use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)!)
4. Create a PR

No need to ask me if you can implement something, if I think it's a good thing I'll merge it in.
