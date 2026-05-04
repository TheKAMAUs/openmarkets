use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct KrakenTickerResponse {
    result: std::collections::HashMap<String, KrakenTickerPair>,
}

#[derive(Debug, Deserialize)]
struct KrakenTickerPair {
    c: Vec<String>, // last trade price = c[0]
}

pub async fn fetch_kraken_btc_price() -> anyhow::Result<f64> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSDT";

    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch Kraken price: {}", e))?;
    
    let body: KrakenTickerResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse Kraken response: {}", e))?;

    let pair_key = body
        .result
        .keys()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No pair data in response"))?;
    
    let ticker = body
        .result
        .get(pair_key)
        .ok_or_else(|| anyhow::anyhow!("Pair data not found"))?;

    let price = ticker.c[0]
        .parse::<f64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse price: {}", e))?;

    Ok(price)
}

// Optional: Multi-symbol support
pub async fn fetch_kraken_prices(symbols: &[&str]) -> anyhow::Result<Vec<(String, f64)>> {
    let symbols_str = symbols.join(",");
    let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", symbols_str);

    let resp = reqwest::get(&url).await?;
    let body: KrakenTickerResponse = resp.json().await?;

    let mut prices = Vec::new();
    for (pair, ticker) in body.result {
        let price = ticker.c[0].parse::<f64>()?;
        prices.push((pair, price));
    }

    Ok(prices)
}