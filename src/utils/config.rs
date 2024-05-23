#![allow(non_snake_case, dead_code)]

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_MAX_FILE_SIZE: usize = 1024 * 1024 * 10;

pub struct Config {
    pub DATABASE_URI: String,
    pub PORT: u16,
    pub MAX_FILE_SIZE: usize,
    pub PRIVATE_TOKEN: String,
}

impl Config {
    pub fn parse() -> Self {
        log::info!("Parsing config...");
        let db_uri = std::env::var("DATABASE_URI").expect("database uri must be provided");
        log::info!("DATABASE_URI: {:?}", db_uri);
        let pvt_token = std::env::var("PRIVATE_TOKEN").expect("private token must be provided");
        log::info!("PRIVATE_TOKEN: {:?}", pvt_token);
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "-".to_string())
            .parse()
            .unwrap_or_else(|_| DEFAULT_PORT);
        log::info!("PORT: {:?}", port);
        let max_file_size = std::env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "-".to_string())
            .parse()
            .unwrap_or_else(|_| DEFAULT_MAX_FILE_SIZE);
        log::info!("MAX_FILE_SIZE: {:?}", max_file_size);
        Self {
            DATABASE_URI: db_uri,
            PORT: port,
            MAX_FILE_SIZE: max_file_size,
            PRIVATE_TOKEN: pvt_token,
        }
    }
}
