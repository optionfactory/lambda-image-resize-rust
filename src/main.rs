extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::num::NonZeroU32;
use std::mem;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use image::{ImageOutputFormat, ImageError};

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

mod config;

use aws_lambda_events::event::s3::{S3Event, S3EventRecord};
use config::Config;
use lambda::handler_fn;
use serde_json::Value;
use smartcrop::{Analyzer, CropSettings};

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    lambda::run(handler_fn(handle_event)).await?;
    Ok(())
}

async fn handle_event(event: Value, _ctx: lambda::Context) -> Result<(), Error> {
    let config = Config::new();
    info!("config is: {:?}", config);
    let s3_event: S3Event =
        serde_json::from_value(event)?;

    info!("event is: {:?}", s3_event);
    for record in s3_event.records {
        handle_record(&config, record);
    }
    Ok(())
}

fn parse_region(region_name: &str) -> Region {
    match region_name {
        "eu-south-1" => Region::Custom { region: "eu-south-1".to_string(), endpoint: "https://s3.eu-south-1.amazonaws.com".to_string() },
        _ => region_name.parse().expect("Could not parse region from record")
    }
}

fn handle_record(config: &Config, record: S3EventRecord) {
    let credentials = Credentials::default().expect("Could not retrieve default credentials");
    let region_name = record
        .aws_region
        .expect("Could not get region from record");
    let region = parse_region(&region_name);
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
    let last_folder = source_path_parts[source_path_parts.len()-2];
    if "originals" != last_folder {
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

    let mut img = image::load_from_memory(&data)
        .ok()
        .expect("Opening image failed");

    let mut dest_path_parts = source_path_parts.clone();
    let _: Vec<_> = config
        .sizes
        .iter()
        .map(|size| {
            let buffer = resize_image(&mut img, &(size.1, size.2)).expect("Could not resize image");
            let last_folder_pos = dest_path_parts.len() - 2;
            let _ = mem::replace(&mut dest_path_parts[last_folder_pos], &size.0);
            
            let dest = dest_path_parts.join("/");
            let (_, code) = bucket
                .put_object_with_content_type_blocking(&dest, &buffer, "image/jpeg")
                .expect(&format!("Could not upload object to :{}", &dest));
            info!("Uploaded: {} with code: {}", &dest, &code);
        })
        .collect();
}

fn resize_image(img: &mut image::DynamicImage, (new_w, new_h): &(u32, u32)) -> Result<Vec<u8>, ImageError> {
    
    let an: Analyzer = Analyzer::new(CropSettings::default());
    let crop_result = an.find_best_crop(
        img,
        NonZeroU32::new(*new_w).unwrap(),
        NonZeroU32::new(*new_h).unwrap(),
    )
    .unwrap();
    let crop = crop_result.crop;
    
    let mut result: Vec<u8> = Vec::new();
    let cropped = img.crop(crop.x, crop.y, crop.width, crop.height);
    let scaled = cropped.resize(*new_w, *new_h, image::imageops::FilterType::Lanczos3);
    scaled.write_to(&mut result, ImageOutputFormat::Png)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let credentials = Credentials::anonymous().unwrap();
        let region: Region = Region::Custom { region: "eu-south-1".to_string(), endpoint: "https://s3.eu-south-1.amazonaws.com".to_string() };
        let bucket = Bucket::new(
            "test-opfa-img-resize",
            region,
            credentials,
        ).expect("Could not get bucket");
        let (_data, _) = bucket
        .get_object_blocking("originals/octane.png")
        .expect(&format!("Could not get object"));
    }
}