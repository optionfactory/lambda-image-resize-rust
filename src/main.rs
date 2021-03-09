#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use simple_logger::SimpleLogger;

use image::{ImageOutputFormat, ImageError};

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

mod config;

use aws_lambda_events::event::s3::{S3Event, S3EventRecord};
use config::Config;
use lambda::error::HandlerError;
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();

    lambda!(handle_event);

    Ok(())
}

fn handle_event(event: Value, _ctx: lambda::Context) -> Result<(), HandlerError> {
    let config = Config::new();

    let s3_event: S3Event =
        serde_json::from_value(event)?;

    for record in s3_event.records {
        handle_record(&config, record);
    }
    Ok(())
}

fn handle_record(config: &Config, record: S3EventRecord) {
    let credentials = Credentials::default().expect("Could not retrieve default credentials");
    let region: Region = record
        .aws_region
        .expect("Could not get region from record")
        .parse()
        .expect("Could not parse region from record");
    let bucket = Bucket::new(
        &record
            .s3
            .bucket
            .name
            .expect("Could not get bucket name from record"),
        region,
        credentials,
    ).expect("Could not get bucket");
    let source = record
        .s3
        .object
        .key
        .expect("Could not get key from object record");
    info!("Fetching: {}, config: {:?}", &source, &config);

    /* Make sure we don't process files twice */
    let source_path_parts: Vec<&str> = source.split("/").collect();
    if &"originals" != source_path_parts.last().unwrap() {
        warn!("Source: '{}' not in originals, skipping.", &source);
        return;
    }
    for size in &config.sizes {
        if "originals" == size.0 {
            panic!("'originals' is not a valid target folder");
        }
    }

    let (data, _) = bucket
        .get_object_blocking(&source)
        .expect(&format!("Could not get object: {}", &source));

    let img = image::load_from_memory(&data)
        .ok()
        .expect("Opening image failed");

    let mut dest_path_parts = source_path_parts.clone();
    let _: Vec<_> = config
        .sizes
        .iter()
        .map(|size| {
            let buffer = resize_image(&img, &(size.1, size.2)).expect("Could not resize image");
            dest_path_parts.pop();
            dest_path_parts.push(&size.0);
            
            let dest = dest_path_parts.join("/");
            let (_, code) = bucket
                .put_object_with_content_type_blocking(&dest, &buffer, "image/jpeg")
                .expect(&format!("Could not upload object to :{}", &dest));
            info!("Uploaded: {} with: {}", &dest, &code);
        })
        .collect();
}

fn resize_image(img: &image::DynamicImage, (new_w, new_h): &(u32, u32)) -> Result<Vec<u8>, ImageError> {
    let mut result: Vec<u8> = Vec::new();
    let scaled = img.resize(*new_w, *new_h, image::imageops::FilterType::Lanczos3);
    scaled.write_to(&mut result, ImageOutputFormat::Png)?;

    Ok(result)
}
