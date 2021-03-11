use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub sizes: Vec<(String, u32,u32)>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sizes: Config::parse_sizes()
        }
    }

    pub fn parse_sizes() -> Vec<(String, u32,u32)> {
        let mut sizes = Vec::new();
        if let Ok(sizes_string) = env::var("SIZES") {
            for size_string in sizes_string.split(',').into_iter() {
                let mut parts = size_string.split(":");
                let folder = parts.next().expect(&sizes_string).to_string();
                let mut dims = parts.next().unwrap().split('x');
                let size = (
                    folder,
                    dims.next().unwrap().parse().unwrap(), 
                    dims.next().unwrap().parse().unwrap());
                sizes.push(size);
            }
        };

        sizes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        env::set_var("SIZES", "@1x:200x300,@2x:300x400");
        let cfg = Config::new();
        assert_eq!(2, cfg.sizes.len());
        assert_eq!(200, (cfg.sizes[0]).1);
    }
}