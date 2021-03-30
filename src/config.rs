use std::env;
use std::str::FromStr;
use std::fmt::Debug;
use num_rational::Rational32;

#[derive(Debug)]
pub struct Config {
    pub dest_region: String,
    pub dest_bucket: String,
    pub sizes: Vec<(String, u32,u32)>,
    pub ratios: Vec<(String, Rational32,Rational32)>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            dest_region: env::var("RESIZE_DEST_REGION").expect("RESIZE_DEST_REGION not set"),
            dest_bucket: env::var("RESIZE_DEST_BUCKET").expect("RESIZE_DEST_BUCKET not set"),
            sizes: Config::parse_sizes(&env::var("RESIZE_SIZES").expect("RESIZE_SIZES not set")),
            ratios: Config::parse_sizes(&env::var("RESIZE_RATIOS").expect("RESIZE_RATIOS not set")),
        }
    }

    pub fn parse_sizes<T: FromStr>(sizes_string: &str) -> Vec<(String, T, T)>
        where <T as FromStr>::Err: Debug  {
        sizes_string.split(',').into_iter()
            .filter(|e| !e.is_empty())
            .map(|size_string| {
                let mut parts = size_string.split(":");
                let folder = parts.next().expect(&sizes_string).to_string();
                let mut dims = parts.next().unwrap().split('x');
                (
                    folder,
                    dims.next().unwrap().parse().unwrap(), 
                    dims.next().unwrap().parse().unwrap()
                )})
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_empty_sizes() {
        env::set_var("RESIZE_DEST_REGION", "eu-south-1");
        env::set_var("RESIZE_DEST_BUCKET", "mario");
        env::set_var("RESIZE_SIZES", "");
        env::set_var("RESIZE_RATIOS", "10x:10x10");
        let cfg = Config::new();
        assert_eq!(0, cfg.sizes.len());
    }
    #[test]
    fn can_parse_sizes() {
        env::set_var("RESIZE_DEST_REGION", "eu-south-1");
        env::set_var("RESIZE_DEST_BUCKET", "mario");
        env::set_var("RESIZE_SIZES", "@1x:200x300,@2x:300x400");
        env::set_var("RESIZE_RATIOS", "10x:10x10");
        let cfg = Config::new();
        assert_eq!(2, cfg.sizes.len());
        assert_eq!(200, (cfg.sizes[0]).1);
    }

    #[test]
    fn fails_without_dest_bucket() {
        env::set_var("RESIZE_DEST_REGION", "eu-south-1");
        env::set_var("RESIZE_SIZES", "@1x:200x300,@2x:300x400");
        env::set_var("RESIZE_RATIOS", "10x:10x10");
        let cfg = Config::new();
        assert_eq!(2, cfg.sizes.len());
        assert_eq!(200, (cfg.sizes[0]).1);
    }
}
