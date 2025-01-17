use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use actix_web::{HttpResponse, Responder, web};
use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::stream::StreamExt;
use futures::TryStreamExt;
use log::{error, info};
use sysinfo::System;

async fn stream_blob(path: web::Path<(String, String)>) -> impl Responder {
    let (container, blob) = path.into_inner();

    return match get_blob_client(&container, &blob).await {
        Ok(client) => {
            let body_stream = async_stream::stream! {
                let client = client.lock().unwrap();
                //let blob = client.get().into_stream().next().await.unwrap().unwrap();
                let mut stream = client.get().into_stream();
                while let Some(value) = stream.next().await {
                    match value {
                    Ok(d) => {
                            let mut data = d.data.into_stream();
                            while let Some(chunk) = data.next().await {
                                match chunk {
                                    Ok(bytes) => yield Ok(web::Bytes::from(bytes)),
                                    Err(e) => yield Err(actix_web::error::ErrorInternalServerError(e)),
                                }
                            }
                        },
                        Err(e) => {
                            yield Err(actix_web::error::ErrorInternalServerError(e));
                        }
                    }
                }
            }
                .map_err(|e| {
                    error!("Error streaming blob: {:?}", e);
                    actix_web::error::ErrorInternalServerError(e)
                });

            HttpResponse::Ok()
                .content_type("application/octet-stream")
                .streaming(Box::pin(body_stream))
        }
        Err(err) => {
            error!("Error getting blob client: {:?}", err);
            HttpResponse::InternalServerError()
                .body(format!("Failed to get blob client: {}", err))
        }
    };
}

async fn get_blob_client(
    container: &String,
    blob: &String,
) -> Result<Arc<Mutex<BlobClient>>, Box<dyn std::error::Error>> {
    let account = std::env::var("STORAGE_ACCOUNT").expect("missing STORAGE_ACCOUNT");
    let access_key = std::env::var("STORAGE_ACCESS_KEY").expect("missing STORAGE_ACCOUNT_KEY");

    info!("Connecting to storage account: {}", account);
    info!("Getting blob: {}/{}", container, blob);

    let storage_credentials = StorageCredentials::access_key(account.clone(), access_key);
    let client = ClientBuilder::new(account, storage_credentials).blob_client(container, blob);

    Ok(Arc::new(Mutex::new(client)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    info!("Starting server");

    let mut system = System::new_all();

    // Refresh system memory information
    system.refresh_memory();

    // Get total and used memory
    info!("Total memory: {:.2} MB , Free memory: {:.2}",
        system.total_memory() / (1024 * 1024),
        system.free_memory() / (1024 * 1024)
    );

    thread::spawn(move || loop {
        system.refresh_all();
        thread::sleep(Duration::from_secs(1));
        info!(
            "CPU: {}% and Free Memory : {:.2} MB",
            system.global_cpu_info().cpu_usage(),
            (system.free_memory() as f64) / (1024 * 1024) as f64
        );


    });

    let _result = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .app_data(web::PayloadConfig::new(usize::MAX)) // Increase payload limit
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .route("/{container}/{blob}", web::get().to(stream_blob))
    })
        .bind("0.0.0.0:8888")?
        .run()
        .await;


    info!("Server stopped");
    Ok(())
}
