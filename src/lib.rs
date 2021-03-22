use std::num::NonZeroU32;
use image::{DynamicImage};
use smartcrop::{Analyzer, CropSettings};

pub fn resize_image(img: &mut image::DynamicImage, (new_w, new_h): &(u32, u32)) -> DynamicImage {
    
    let an: Analyzer = Analyzer::new(CropSettings::default());
    let crop_result = an.find_best_crop(
        img,
        NonZeroU32::new(*new_w).unwrap(),
        NonZeroU32::new(*new_h).unwrap(),
    )
    .unwrap();
    let crop = crop_result.crop;
    let cropped = img.crop(crop.x, crop.y, crop.width, crop.height);
    let scaled = cropped.resize(*new_w, *new_h, image::imageops::FilterType::Lanczos3);
    scaled
}