use lambda_image_resize_rust::resize_image;
use clap::{App, Arg};
use std::fs::File;
use std::io::Write;

fn main() {
    let matches = App::new("resize")
        .arg(Arg::with_name("INPUT").required(true))
        .arg(Arg::with_name("OUTPUT").required(true))
        .arg(Arg::with_name("WIDTH").required(true))
        .arg(Arg::with_name("HEIGHT").required(true))
        .get_matches();

    let file_in = matches.value_of("INPUT").unwrap();
    let file_out = matches.value_of("OUTPUT").unwrap();

    let mut img = image::open(file_in).unwrap();
    let data = resize_image(
        &mut img,
        &(matches.value_of("WIDTH").unwrap().parse().unwrap(), matches.value_of("HEIGHT").unwrap().parse().unwrap()))
        .unwrap();
    let mut file = File::create(file_out).unwrap();
    file.write_all(&data).unwrap();
}