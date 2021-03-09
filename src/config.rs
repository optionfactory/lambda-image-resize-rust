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
                let folder = parts.next().unwrap().to_string();
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
