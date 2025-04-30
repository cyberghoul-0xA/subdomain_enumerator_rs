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
    let file = match File::open("subdomains.txt"){
        Ok(file) => file,
        Err(err) => {
            eprintln!("[-] Failed to open subdomains.txt: {}",err);
            std::process::exit(1);
        }
    };
    let reader = BufReader::new(file);
    let arg: Vec<String> = args().collect();
    if arg.len() < 2{
        eprintln!("Usage: cargo run <domain_name> in {}",arg[0]);
        std::process::exit(1);
    }

    let domain = &arg[1];
    let client = Client::new();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("valid.txt").await?;
        
    let output = Arc::new(Mutex::new(file));
    let mut handles = vec![];

    for line in reader.lines() {
        if let Ok(subdomain) = line {
            let client = client.clone();
            let output = Arc::clone(&output);
            let subdomain = subdomain.trim().to_string();
            let full_domain = format!("{}.{}", subdomain,domain);

            let handle = task::spawn(async move {
                check_subdomain(client, output,full_domain).await;
            });

               

            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn check_subdomain(client: Client, output:Arc<Mutex<tokio::fs::File>>, full_domain: String){
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
                            println!("[-] {} Response:  {}", url, resp.status());
                        }
                    }
                    Err(e) => {
                        println!("[-] Failed to reach {}: {}", url, e);
                    }
                }
            }
        } else {
            println!("[DNS] {} No address Returned", full_domain);
        }
    } else {
        println!("[DNS] {} failed to resolve", full_domain);
    }
}

