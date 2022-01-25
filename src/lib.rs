use std::io::{BufRead, Seek};
use std::num::NonZeroU32;
use image::{DynamicImage};
use smartcrop::{Analyzer, CropSettings};

pub fn resize_image(img: &image::DynamicImage, (new_w, new_h): (u32, u32)) -> DynamicImage {
    
    let an: Analyzer = Analyzer::new(CropSettings::default());
    let crop_result = an.find_best_crop(
        img,
        NonZeroU32::new(new_w).unwrap(),
        NonZeroU32::new(new_h).unwrap(),
    )
    .unwrap();
    let crop = crop_result.crop;
    let cropped = img.crop_imm(crop.x, crop.y, crop.width, crop.height);
    let scaled = cropped.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
    scaled
}

pub fn rotation_for<R: BufRead + Seek>(container: &mut R) -> Option<fn(&DynamicImage) -> DynamicImage> {
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(container).unwrap();
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY).and_then(|orientation: &exif::Field|
        match orientation.value.get_uint(0) {
            Some(8) => Some(DynamicImage::rotate270 as fn(&DynamicImage) -> DynamicImage),
            Some(6) => Some(DynamicImage::rotate90 as fn(&DynamicImage) -> DynamicImage),
            Some(3) => Some(DynamicImage::rotate180 as fn(&DynamicImage) -> DynamicImage),
            Some(1) => None,
            v => panic!("Orientation value is broken: {:?}", v),
        })
}