use anyhow::Context;
use anyhow::Result;
use skia_safe::image_filters::drop_shadow_only;
use skia_safe::EncodedImageFormat;
use skia_safe::Surface;
use skia_safe::{Color, Data, IRect, Image, Point, RRect, Rect};

use std::io::{self, Read, Write};

fn new_canvas(width: i32, height: i32) -> Surface {
    let mut surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");
    surface.canvas().clear(Color::TRANSPARENT);
    surface
}

fn main() -> Result<()> {
    let mut user_input: Vec<u8> = Vec::new();
    let mut stdin = io::stdin();
    stdin
        .read_to_end(&mut user_input)
        .context("Failed to read image from stdin")?;

    let img: Image = unsafe {
        Image::from_encoded(Data::new_bytes(&user_input)).context("Failed to decode image")?
    };

    let x_padding = img.width() / 6;
    let y_padding = img.height() / 6;

    let padding = x_padding.max(y_padding);
    let blur = x_padding.max(y_padding);

    let radius = 12.0;

    let mut canvas = new_canvas(img.width() + padding, img.height() + padding);

    canvas.canvas().clip_rrect(
        RRect::new_rect_radii(
            Rect {
                left: (padding / 2) as f32,
                top: (padding / 2) as f32,
                bottom: (img.height() + padding / 2) as f32,
                right: (img.width() + padding / 2) as f32,
            },
            &[
                Point::new(radius, radius),
                Point::new(radius, radius),
                Point::new(radius, radius),
                Point::new(radius, radius),
            ],
        ),
        None,
        true,
    );

    canvas
        .canvas()
        .draw_image(img, (padding / 2, padding / 2), None);

    let padded_image = canvas.image_snapshot();

    let filter = drop_shadow_only(
        (0, padding / 8),
        (blur as f32 / 6.0, blur as f32 / 6.0),
        Color::BLACK.with_a(100),
        None,
        None,
    )
    .context("Failed to create drop shadow")?;

    let shadow_image = padded_image
        .new_with_filter(
            &filter,
            IRect {
                left: 0,
                top: 0,
                right: padded_image.width(),
                bottom: padded_image.height(),
            },
            IRect {
                left: 0,
                top: 0,
                right: padded_image.width(),
                bottom: padded_image.height(),
            },
        )
        .context("Failed to add drop shadow")?
        .0;

    let mut canvas = new_canvas(padded_image.width(), padded_image.height());

    canvas.canvas().draw_image(shadow_image, (0, 0), None);

    canvas.canvas().draw_image(padded_image, (0, 0), None);

    let image = canvas.image_snapshot();
    let data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .context("Failed to encode image")?;

    let bytes = data.as_bytes();
    let mut stdout = io::stdout();

    stdout.write_all(bytes).context("Failed to output image")?;

    Ok(())
}
