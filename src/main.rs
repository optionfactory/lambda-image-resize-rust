use std::io::{Seek, SeekFrom};
use lambda_runtime as lambda;
use log::{info};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
mod config;
use aws_lambda_events::event::s3::{S3Event, S3EventRecord};
use config::Config;
use lambda::handler_fn;
use serde_json::Value;

use lambda_image_resize_rust::{rotation_for, SmarpCropper};
use image::{GenericImageView, ImageOutputFormat};
use num_rational::Rational32;
use num_traits::cast::ToPrimitive;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_utc_timestamps().with_level(LevelFilter::Info).init().unwrap();
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

const APPLICATION_OCTET_STREAM: &'static str = "application/octet-stream";

fn handle_record(config: &Config, record: S3EventRecord) {
    let region_name = record
        .aws_region
        .expect("Could not get region from record");
    let region = parse_region(&region_name);
    let source_bucket_name = &record
        .s3
        .bucket
        .name
        .expect("Could not get bucket name from record");
    let source_bucket = Bucket::new(
        source_bucket_name,    
        region,
        Credentials::default().expect("Could not retrieve default credentials"),
    ).expect("Could not source bucket");
    let source = record
        .s3
        .object
        .key
        .expect("Could not get key from object record");
    info!("Fetching: {}, config: {:?}", &source, &config);

    let (data, _) = source_bucket
        .get_object_blocking(&source)
        .expect(&format!("Could not get object: {}", &source));
    let (metadata, _) = source_bucket.head_object_blocking(&source).expect(&format!("Could not get object metadata: {}", &source));

    let source_path_parts: Vec<&str> = source.split("/").collect();
    if config.dest_bucket == *source_bucket_name {
        panic!("source and target bucket names are the same. Quitting to avoid an infinite loop.")
    }
    let dest_region = parse_region(&config.dest_region);
    let dest_bucket = Bucket::new(
        &config.dest_bucket,
        dest_region,
        Credentials::default().expect("Could not retrieve default credentials"),
    ).expect("Could not dest bucket");

    let dest_path = |folder: &String| {
        let mut dest_path_parts = source_path_parts.clone();
        dest_path_parts.insert(source_path_parts.len() - 1, &folder);
        dest_path_parts.join("/")
    };

    let mut cursor = std::io::Cursor::new(data);
    let rotate_fn = rotation_for(&mut cursor);
    cursor.seek(SeekFrom::Start(0)).unwrap();
    let reader = image::io::Reader::new(cursor)
        .with_guessed_format()
        .unwrap();
    let format = reader.format();
    let output_format = format.map(ImageOutputFormat::from).unwrap_or(ImageOutputFormat::Unsupported("No input format".to_owned()));

    if format.is_none() || matches!(output_format, ImageOutputFormat::Unsupported(_)) {
        info!("Unsupported format, copying source to destinations without processing");
        let data = reader.into_inner().into_inner();
        let dest_paths = config.sizes.iter().map(|(folder, _,_)| folder)
            .chain(config.ratios.iter().map(|(folder, _,_)| folder))
            .map(dest_path)
            .collect::<Vec<_>>();
        for dest_path in dest_paths {
            let (_, code) = dest_bucket
                .put_object_with_content_type_blocking(&dest_path, &data, metadata.content_type.as_ref().map(String::as_str).unwrap_or(APPLICATION_OCTET_STREAM))
                .expect(&format!("Could not upload object to :{}", &dest_path));
            info!("Uploaded: {} with code: {}", &dest_path, &code);
            }
        return;
    }
    let format = format.unwrap();
    let img = reader
        .decode()
        .expect("Opening image failed");
    let img = rotate_fn.map(|f| f(&img)).unwrap_or(img);
    let targets = config.sizes.iter()
        .cloned()
        .chain(config.ratios.iter()
            .cloned()
            .map(|(folder, wr, hr)| {
                let w = (Rational32::from_integer(img.width() as i32) * wr).trunc().to_u32().unwrap();
                let h = (Rational32::from_integer(img.height() as i32) * hr).trunc().to_u32().unwrap();
                (folder, w, h)
            }))
        .collect::<Vec<_>>();

    let max_w = targets.iter().map(|(_, w, _)| w).max().cloned().unwrap_or(img.width());
    let max_h = targets.iter().map(|(_, _, h)| h).max().cloned().unwrap_or(img.height());

    let mut smart_cropper = SmarpCropper::new(img, Some((max_w, max_h)));
    for (folder, w, h) in targets {
        let cropped_image = smart_cropper.crop(w, h);
        let mut buffer = Vec::new();
        cropped_image.write_to(&mut buffer, format).unwrap();

        let mut dest_path_parts = source_path_parts.clone();
        dest_path_parts.insert(dest_path_parts.len() - 1, &folder);
        let dest = dest_path_parts.join("/");

        let content_type = match format {
            image::ImageFormat::Png => "image/png",
            image::ImageFormat::Jpeg => "image/jpeg",
            image::ImageFormat::Gif => "image/gif",
            _ => APPLICATION_OCTET_STREAM,
        };

        let (_, code) = dest_bucket
            .put_object_with_content_type_blocking(&dest, &buffer, content_type)
            .expect(&format!("Could not upload object to :{}", &dest));
        info!("Uploaded: {} with code: {}", &dest, &code);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_eu_south() {
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
