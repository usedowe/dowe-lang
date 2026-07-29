use icns::{IconFamily, IconType, Image};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().expect("input"));
    let output = PathBuf::from(args.next().expect("output"));
    let mut family = IconFamily::new();
    for size in [16, 32, 64, 128, 256, 512, 1024] {
        let data = fs::read(input.join(format!("{size}.png"))).expect("png");
        let image = Image::read_png(Cursor::new(data)).expect("image");
        let icon_type = IconType::from_pixel_size(size, size).expect("icon type");
        family.add_icon_with_type(&image, icon_type).expect("add");
    }
    let mut data = Vec::new();
    family.write(&mut data).expect("encode");
    fs::write(output, data).expect("write");
}
