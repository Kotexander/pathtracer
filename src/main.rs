mod math;
mod renderer;

fn main() {
    let mut renderer = renderer::Renderer::new();
    renderer.render();

    let img = image::ImageBuffer::from_fn(
        renderer.image.width() as u32,
        renderer.image.height() as u32,
        |x, y| {
            let p = renderer.image.get(x as usize, y as usize);
            image::Rgb([p.x, p.y, p.z])
        },
    );
    let img = image::DynamicImage::ImageRgb32F(img).into_rgb8();

    img.save("Renders/image.png").unwrap();
}
