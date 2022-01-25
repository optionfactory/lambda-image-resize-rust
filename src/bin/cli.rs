use std::io::BufReader;
use lambda_image_resize_rust::{resize_image, rotation_for};
use clap::{App, Arg};

fn main() {
    let matches = App::new("resize")
        .arg(Arg::with_name("INPUT").required(true).index(1))
        .arg(Arg::with_name("OUTPUT").required(true).index(2))
        .arg(Arg::with_name("WIDTH").required(true).short("w").takes_value(true))
        .arg(Arg::with_name("HEIGHT").required(true).short("h").takes_value(true))
        .get_matches();

    let file_in = matches.value_of("INPUT").unwrap();
    let file_out = matches.value_of("OUTPUT").unwrap();
    let rotate_fn = rotation_for(&mut BufReader::new(std::fs::File::open(file_in).unwrap()));
    let reader = image::io::Reader::open(file_in)
        .unwrap()
        .with_guessed_format()
        .unwrap();
    let format = reader.format().unwrap();
    let img = reader.decode().unwrap();
    let img = rotate_fn.map(|f| f(&img)).unwrap_or(img);
    let resized = resize_image(
        &img,
        (matches.value_of("WIDTH").unwrap().parse().unwrap(), matches.value_of("HEIGHT").unwrap().parse().unwrap()));
    resized.save_with_format(file_out, format).unwrap();
}