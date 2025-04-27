use reqwest::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::{net::lookup_host, sync::Mutex, task};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::env::args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("subdomains.txt")?;
    let reader = BufReader::new(file);
    let arg: Vec<String> = args().collect();
    let domain = &arg[1];


    let client = Client::new();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("valid.txt")
        .await?;
    let output = Arc::new(Mutex::new(file));

    let mut handles = vec![];

    for line in reader.lines() {
        if let Ok(subdomain) = line {
            let client = client.clone();
            let output = Arc::clone(&output);
            let subdomain = subdomain.trim().to_string();
            let full_domain = format!("{}.{}", subdomain,domain);

            let handle = task::spawn(async move {
                if let Ok(mut addrs) = lookup_host((full_domain.as_str(), 80)).await {
                    if addrs.next().is_some() {
                        println!("[DNS] {} resolved", full_domain);

                        for scheme in &["http", "https"] {
                            let url = format!("{}://{}", scheme, full_domain);
                            match client.get(&url).send().await {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        println!("[+] Reachable: {} ({})", url, resp.status());
                                        let mut file = output.lock().await;
                                        let _ = file.write_all(format!("{}\n", url).as_bytes()).await;
                                    } else {
                                        println!("[-] {} responded with {}", url, resp.status());
                                    }
                                }
                                Err(e) => {
                                    println!("[-] Failed to reach {}: {}", url, e);
                                }
                            }
                        }
                    } else {
                        println!("[DNS] {} did not return any address", full_domain);
                    }
                } else {
                    println!("[DNS] {} failed to resolve", full_domain);
                }
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
