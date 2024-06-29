use std::fs::File;
use std::io::Write;

use futures::StreamExt;
use log::{error, info};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    info!("Downloading large file");

    let url = "http://localhost:8888/testdata/temp_5GB_file.dat";
    let response = Client::new().get(url).send().await.unwrap();
    let file = File::create("data_5GB.dat");

    match file {
        Ok(mut file) => {
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.unwrap();
                file.write_all(&chunk).unwrap();
            }
        }
        Err(e) => {
            error!("Error creating file: {:?}", e);
        }
    }


    Ok(())
}
