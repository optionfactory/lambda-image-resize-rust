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

use lambda_image_resize_rust::resize_image;

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

    let reader = image::io::Reader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .unwrap();
    let format = reader.format().unwrap();
    let mut img = reader
        .decode()
        .expect("Opening image failed");

    let source_path_parts: Vec<&str> = source.split("/").collect();

    if  config.dest_bucket == *source_bucket_name {
        panic!("source and target bucket names are the same. Quitting to avoid an infinite loop.")
    }
    let dest_region = parse_region(&config.dest_region);
    let dest_bucket = Bucket::new(
        &config.dest_bucket,
        dest_region,
        Credentials::default().expect("Could not retrieve default credentials"),
    ).expect("Could not dest bucket");



    let _: Vec<_> = config
        .sizes
        .iter()
        .map(|size| {
            let resized = resize_image(&mut img, &(size.1, size.2));
            let mut buffer = Vec::new();
            resized.write_to(&mut buffer, format).unwrap();

            let mut dest_path_parts = source_path_parts.clone();
            dest_path_parts.insert(dest_path_parts.len() - 1, &size.0);
            let dest = dest_path_parts.join("/");

            let content_type = match format {
                image::ImageFormat::Png => "image/png",
                image::ImageFormat::Jpeg => "image/jpeg",
                image::ImageFormat::Gif => "image/gif",
                _ => "application/octet-stream"
            };

            let (_, code) = dest_bucket
                .put_object_with_content_type_blocking(&dest, &buffer, content_type)
                .expect(&format!("Could not upload object to :{}", &dest));
            info!("Uploaded: {} with code: {}", &dest, &code);
        })
        .collect();
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
