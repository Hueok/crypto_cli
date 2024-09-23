use reqwest;
use tokio;
use std::error::Error;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc, TimeZone, FixedOffset};
use std::collections::HashMap;
use yansi::Paint;

#[derive(Debug, Serialize, Deserialize)]
struct DataPoint {
    timestamp: u64,
    low: f64,
    high: f64,
    open: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticker {
    ask: String,
    bid: String,
    volume: String,
    trade_id: u64,
    price: String,
    size: String,
    time: String,
    rfq_volume: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Parse JSON
    let parsed_data: Vec<DataPoint> = fetch_data().await?;

    // UTC+9 offset (Korean Standard Time)
    let utc_plus_9 = FixedOffset::east_opt(9 * 3600).expect("Invalid offset");

    /*
    for point in &parsed_data {
        println!("{:?}", point);
    }
        */
    let current:f64 = getLivePrice().await?;

    // Test
    // println!("test current : {}", current);
    // println!("{:?}", parsed_data[1]);
    // println!("{:?}", parsed_data[6]);
    // println!("{:?}", parsed_data[16]);
    // println!("{:?}", parsed_data[31]);
    // println!("{:?}", parsed_data[61]);

    let test: &str = getChangeInfo(&parsed_data, current);

    Ok(())
}

async fn fetch_data() -> Result<Vec<DataPoint>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("User-Agent", "crypto_cli".parse()?);

    let request = client.request(reqwest::Method::GET, "https://api.exchange.coinbase.com/products/BTC-USD/candles?granularity=60")
        .headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;

    // Parse JSON
    let parsed_data: Vec<DataPoint> = serde_json::from_str(&body).expect("Failed to parse JSON in CANDLES");

    Ok(parsed_data)
}

async fn getLivePrice() -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    let request = client.request(reqwest::Method::GET, "https://api.exchange.coinbase.com/products/BTC-USD/ticker")
        .headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;

    // Parse JSON
    let parsed_data: Ticker = serde_json::from_str(&body).expect("Failed to parse JSON in TICKER");
    let price = parsed_data.price.parse::<f64>().unwrap();

    Ok(price)
}

//granularity can be {60, 300, 900, 3600, 21600, 86400} == {1m, 5m, 15m, 1h, 6h, 1d}
fn getChangeInfo(data: &Vec<DataPoint>, current: f64) -> &str {

    let now: &DataPoint = &data[0];
    let mut element: Vec<String> = Vec::new();

    //set gradualarity: test with 5m
    let granularity = "1m";

    //initialize the granularity map
    let mut granularity_map: HashMap<&str, Vec<(usize, &str)>> = HashMap::new();
    granularity_map.insert("1m", vec![(1, " 1m"), (6, " 5m"), (16, "15m"), (31, "30m"), (61, " 1h")]);
    granularity_map.insert("5m", vec![(1, " 5m"), (6, "30m"), (12, " 1h"), (144, "12h"), (287, " 1d")]);
    granularity_map.insert("15m", vec![(1, "15m"), (2, "30m"), (4, " 1h"), (8, " 2h"), (16, " 4h")]);
    granularity_map.insert("1h", vec![(1, " 1h"), (2, " 2h"), (5, " 5h"), (12, "12h"), (24, " 1d")]);
    granularity_map.insert("6h", vec![(1, " 6h"), (2, "12h"), (4, " 1d"), (8, " 2d"), (16, " 4d")]);
    granularity_map.insert("1d", vec![(1, " 1d"), (2, " 2d"), (7, " 1w"), (14, " 2w"), (28, " 1m")]);

    // Example usage
    // for (key, value) in &granularity_map {
    //     println!("{}: {:?}", key, value);
    // }

    for i in 0..=4{
        let idx = granularity_map.get(granularity).unwrap()[i].0;
        let last_price = data[idx].close;
        let change = getChange(last_price, current);
        element.push(change);
    }

    print!("BTC-USD | ${}\t", current);
    for i in 0..=4{
        print!("{}: {}\t", granularity_map.get(granularity).unwrap()[i].1, element[i]);
    }
    println!();

    "test"
}

fn getChange(before: f64, after: f64) -> String {
    let change = (after - before) / before * 100.0;
    let rounded_change = (change* 100.0).round() / 100.0;
    if rounded_change > 0.0 {
        Paint::green(format!("+{:.2}%", rounded_change)).to_string()
    } else if rounded_change < 0.0 {
        Paint::red(format!("{:.2}%", rounded_change)).to_string()
    } else {
        Paint::white(format!("{:.2}%", rounded_change)).to_string()
    }
}