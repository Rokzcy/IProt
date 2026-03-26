// Import required crates for HTTP requests, async runtime, I/O, random number generation, and collections
use reqwest::Client;
use tokio::time::{interval, Duration};
use std::io::{self, BufRead};
use rand::Rng;
use std::collections::VecDeque;

#[tokio::main]  // Makes main function async with Tokio runtime
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prompt user for rotation interval in seconds
    println!("IP Rotator for Pentesting - Enter interval in seconds (e.g., 30):");
    
    // Read from stdin line by line
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    // Get first line (interval) and parse as u64
    let interval_str = lines.next().unwrap()?.trim().to_string();
    let interval_secs: u64 = interval_str.parse()?;
    
    // Prompt user for proxy list
    println!("Interval set to {} seconds. Enter proxy servers (one per line, format 'http://ip:port', empty line to start):", interval_secs);
    
    // Collect proxies into Vec until empty line
    let mut proxies = Vec::new();
    while let Some(line) = lines.next() {
        let proxy = line?.trim().to_string();
        if proxy.is_empty() {  // Break on empty line
            break;
        }
        proxies.push(proxy);  // Add proxy to list
    }
    
    // Validate at least one proxy provided
    if proxies.is_empty() {
        println!("No proxies provided. Exiting.");
        return Ok(());
    }
    
    // Confirm setup and start message
    println!("Loaded {} proxies. Cycling through them every {} seconds.", proxies.len(), interval_secs);
    println!("Starting IP rotation loop (Ctrl+C to stop)...");
    
    // Configure HTTP client with connection pooling and timeouts
    let client_builder = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))  // Close idle connections after 30s
        .timeout(Duration::from_secs(10));           // 10s timeout per request
    
    // Create circular queue from proxy list for rotation
    let mut proxy_queue = VecDeque::from(proxies);
    
    // Initialize random number generator for proxy selection
    let mut rng = rand::thread_rng();
    
    // Create timer that ticks every N seconds
    let mut interval = interval(Duration::from_secs(interval_secs));
    
    // Infinite loop - rotate proxies forever until Ctrl+C
    loop {
        interval.tick().await;  // Wait for next interval tick
        
        // Get random index from current queue size
        let proxy_count = proxy_queue.len();
        let random_index = rng.gen_range(0..proxy_count);
        
        // Clone current proxy for use (borrowing rules)
        let current_proxy = proxy_queue[random_index].clone();
        
        // Log which proxy is being activated
        println!("🔄 Rotating to proxy: {}", current_proxy);
        
        // Test proxy and get new IP (or handle failure)
        match test_proxy(&current_proxy).await {
            Ok(ip) => println!("✅ New IP: {} via {}", ip, current_proxy),
            Err(e) => println!("❌ Proxy failed {}: {}", current_proxy, e),
        }
        
        // Rotate queue: remove selected proxy and add to end (circular shift)
        let proxy = proxy_queue.remove(random_index).unwrap();
        proxy_queue.push_back(proxy);
    }
}

// Test function: sends request through proxy and returns external IP
async fn test_proxy(proxy_url: &str) -> Result<String, reqwest::Error> {
    // Create new client configured with this specific proxy
    let client = Client::builder()
        .proxy(reqwest::Proxy::http(proxy_url)?)  // Set HTTP proxy for this client
        .build()?;
    
    // Send GET request to IP checker service
    let response = client
        .get("http://httpbin.org/ip")  // Public IP detection endpoint
        .send()
        .await?;
    
    // Define struct matching JSON response format
    #[derive(serde::Deserialize)]
    struct IpResponse {
        origin: String,  // Field containing the external IP
    }
    
    // Parse JSON response into struct
    let ip_info: IpResponse = response.json().await?;
    
    // Return the detected IP address
    Ok(ip_info.origin)
}
