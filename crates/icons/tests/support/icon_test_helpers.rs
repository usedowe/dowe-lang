fn fixture() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("assets")).expect("assets");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(
        temp.path().join("assets/icon.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100"><path fill="#ffffff" d="M0 0h200v100H0z"/></svg>"##,
    )
    .expect("svg");
    temp
}

fn png_dimensions(path: &std::path::Path) -> (u32, u32) {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path).expect("png")));
    let reader = decoder.read_info().expect("valid png");
    (reader.info().width, reader.info().height)
}

fn png_rgba(path: &std::path::Path) -> Vec<u8> {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path).expect("png")));
    let mut reader = decoder.read_info().expect("valid png");
    let mut output = vec![0; reader.output_buffer_size().expect("buffer size")];
    let info = reader.next_frame(&mut output).expect("frame");
    assert_eq!(info.color_type, png::ColorType::Rgba);
    output.truncate(info.buffer_size());
    output
}

fn pixel(data: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    data[offset..offset + 4].try_into().expect("pixel")
}

struct PixelBounds {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

impl PixelBounds {
    fn width(&self) -> usize {
        self.max_x - self.min_x + 1
    }

    fn height(&self) -> usize {
        self.max_y - self.min_y + 1
    }

    fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) as f32 * 0.5
    }

    fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) as f32 * 0.5
    }
}

fn opaque_bounds(data: &[u8], width: usize, predicate: impl Fn(&[u8]) -> bool) -> PixelBounds {
    let mut bounds = PixelBounds {
        min_x: width,
        min_y: width,
        max_x: 0,
        max_y: 0,
    };
    for (index, value) in data.chunks_exact(4).enumerate() {
        if predicate(value) && value[3] > 8 {
            let x = index % width;
            let y = index / width;
            bounds.min_x = bounds.min_x.min(x);
            bounds.min_y = bounds.min_y.min(y);
            bounds.max_x = bounds.max_x.max(x);
            bounds.max_y = bounds.max_y.max(y);
        }
    }
    assert!(bounds.min_x <= bounds.max_x);
    bounds
}
