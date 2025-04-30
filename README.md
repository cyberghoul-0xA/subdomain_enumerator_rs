# Subdomain Enumeration in Rust

This is a simple and concurrent subdomain enumeration tool written in Rust. It reads subdomains from a file, resolves DNS, checks HTTP(S) reachability, and saves valid endpoints.

## Features

- Asynchronous DNS resolution using `tokio`
- HTTP and HTTPS validation with `reqwest`
- Output of valid URLs to a file (`valid.txt`)
- Lightweight and fast(`RUST`)

## Requirements 

- Rust and Cargo

## Dependancies

- tokio
- reqwest

## Usage

1. Clone the repo:
2. ```cd subdomain_enumerator_rs ```
3. Prepare the subdomains.txt file containing the subdomains
4. Run the tool <pre> ```cargo run <domain_name> ``` </pre>

 
