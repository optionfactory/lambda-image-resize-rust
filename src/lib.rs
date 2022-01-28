use std::io::{BufRead, Seek};
use std::num::NonZeroU32;
use image::{DynamicImage, GenericImageView};
use smartcrop::{Analyzer, CropSettings, Crop};
use std::collections::HashMap;
use image::imageops::FilterType;

pub struct SmarpCropper {
    analyzer: Analyzer,
    max_bounds: Option<(u32, u32)>,
    img: DynamicImage,
    crops_by_scale: HashMap<u64, Crop>,
}

impl SmarpCropper {
    pub fn new(mut img: image::DynamicImage, mut max_bounds: Option<(u32, u32)>) -> Self {
        if let Some((max_w, max_h)) = max_bounds {
            if img.width() > 4 * max_w && img.height() > 4 * max_h {
                img = img.resize(4 * max_w, 4 * max_h, FilterType::Triangle);
            } else {
                max_bounds = None;
            }
        }

        SmarpCropper {
            analyzer: Analyzer::new(CropSettings::default()),
            max_bounds,
            img,
            crops_by_scale: HashMap::default(),
        }
    }

    pub fn crop(&mut self, w: u32, h: u32) -> DynamicImage {
        if let Some((max_w, max_h)) = self.max_bounds {
            assert!(w <= max_w && h <= max_h);
        }
        let scale = (w as f64 / h as f64 * 1000000.0) as u64;
        let img = &self.img;
        let analyzer = &self.analyzer;
        let crop = self.crops_by_scale.entry(scale).or_insert_with(|| {
            analyzer.find_best_crop(
                img,
                NonZeroU32::new(w).unwrap(),
                NonZeroU32::new(h).unwrap(),
            )
                .unwrap().crop
        });
        let cropped = self.img.crop_imm(crop.x, crop.y, crop.width, crop.height);
        let scaled = cropped.resize(w, h, image::imageops::FilterType::Triangle);
        scaled
    }
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