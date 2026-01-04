use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;

#[derive(Debug)]
struct ReferenceData {
    product_type: String,
    exchange: String,
    exchange_symbol: String,
    tick_size: String,
    lot_size: String,
}

// Binance Spot structures
#[derive(Debug, Deserialize)]
struct BinanceSpotExchangeInfo {
    symbols: Vec<BinanceSpotSymbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceSpotSymbol {
    symbol: String,
    filters: Vec<BinanceFilter>,
}

// Binance Futures structures
#[derive(Debug, Deserialize)]
struct BinanceFuturesExchangeInfo {
    symbols: Vec<BinanceFuturesSymbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceFuturesSymbol {
    symbol: String,
    filters: Vec<BinanceFilter>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
enum BinanceFilter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter { tickSize: String },
    #[serde(rename = "LOT_SIZE")]
    LotSize { stepSize: String },
    #[serde(other)]
    Other,
}

// OKX structures
#[derive(Debug, Deserialize)]
struct OkxResponse {
    data: Vec<OkxInstrument>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OkxInstrument {
    inst_id: String,
    tick_sz: String,
    lot_sz: String,
}

const SYMBOLS: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT", "LINKUSDT", "BNBUSDT", "AVAXUSDT"];

#[tokio::main]
async fn main() -> Result<()> {
    println!("Fetching reference data from exchanges...");

    let mut all_data = Vec::new();

    // Fetch Binance data
    all_data.extend(fetch_binance_spot().await?);
    all_data.extend(fetch_binance_futures().await?);

    // Fetch OKX data
    all_data.extend(fetch_okx_spot().await?);
    all_data.extend(fetch_okx_futures().await?);

    println!("Fetched {} records", all_data.len());

    // Save to SQLite
    save_to_sqlite(all_data)?;

    println!("Data saved successfully!");
    Ok(())
}

async fn fetch_binance_spot() -> Result<Vec<ReferenceData>> {
    let url = "https://api.binance.com/api/v3/exchangeInfo";
    let client = reqwest::Client::new();
    let response: BinanceSpotExchangeInfo = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    let mut results = Vec::new();
    for symbol_info in response.symbols {
        if !SYMBOLS.contains(&symbol_info.symbol.as_str()) {
            continue;
        }

        let mut tick_size = String::new();
        let mut lot_size = String::new();

        for filter in symbol_info.filters {
            match filter {
                BinanceFilter::PriceFilter { tickSize } => tick_size = tickSize,
                BinanceFilter::LotSize { stepSize } => lot_size = stepSize,
                _ => {}
            }
        }

        results.push(ReferenceData {
            product_type: "spot".to_string(),
            exchange: "binance".to_string(),
            exchange_symbol: symbol_info.symbol,
            tick_size,
            lot_size,
        });
    }

    Ok(results)
}

async fn fetch_binance_futures() -> Result<Vec<ReferenceData>> {
    let url = "https://fapi.binance.com/fapi/v1/exchangeInfo";
    let client = reqwest::Client::new();
    let response: BinanceFuturesExchangeInfo = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    let mut results = Vec::new();
    for symbol_info in response.symbols {
        if !SYMBOLS.contains(&symbol_info.symbol.as_str()) {
            continue;
        }

        let mut tick_size = String::new();
        let mut lot_size = String::new();

        for filter in symbol_info.filters {
            match filter {
                BinanceFilter::PriceFilter { tickSize } => tick_size = tickSize,
                BinanceFilter::LotSize { stepSize } => lot_size = stepSize,
                _ => {}
            }
        }

        results.push(ReferenceData {
            product_type: "perp".to_string(),
            exchange: "binance".to_string(),
            exchange_symbol: symbol_info.symbol,
            tick_size,
            lot_size,
        });
    }

    Ok(results)
}

async fn fetch_okx_spot() -> Result<Vec<ReferenceData>> {
    let url = "https://www.okx.com/api/v5/public/instruments?instType=SPOT";
    let client = reqwest::Client::new();
    let response: OkxResponse = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    let mut results = Vec::new();
    for inst in response.data {
        let normalized = inst.inst_id.replace("-", "");
        if !SYMBOLS.contains(&normalized.as_str()) {
            continue;
        }

        results.push(ReferenceData {
            product_type: "spot".to_string(),
            exchange: "okx".to_string(),
            exchange_symbol: inst.inst_id,
            tick_size: inst.tick_sz,
            lot_size: inst.lot_sz,
        });
    }

    Ok(results)
}

async fn fetch_okx_futures() -> Result<Vec<ReferenceData>> {
    let url = "https://www.okx.com/api/v5/public/instruments?instType=SWAP";
    let client = reqwest::Client::new();
    let response: OkxResponse = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    let mut results = Vec::new();
    for inst in response.data {
        let normalized = inst.inst_id.replace("-", "");
        if !SYMBOLS.contains(&normalized.as_str()) {
            continue;
        }

        results.push(ReferenceData {
            product_type: "perp".to_string(),
            exchange: "okx".to_string(),
            exchange_symbol: inst.inst_id,
            tick_size: inst.tick_sz,
            lot_size: inst.lot_sz,
        });
    }

    Ok(results)
}

fn save_to_sqlite(data: Vec<ReferenceData>) -> Result<()> {
    // Create or open the SQLite database file
    let conn = Connection::open("crypto_refdata.db")
        .context("Failed to open SQLite database")?;

    // Create table if not exists
    conn.execute(
        r"CREATE TABLE IF NOT EXISTS reference_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_type TEXT NOT NULL,
            exchange TEXT NOT NULL,
            exchange_symbol TEXT NOT NULL,
            tick_size TEXT NOT NULL,
            lot_size TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(product_type, exchange, exchange_symbol)
        )",
        [],
    )?;

    // Insert or update data
    for item in data {
        conn.execute(
            r"INSERT INTO reference_data 
              (product_type, exchange, exchange_symbol, tick_size, lot_size)
              VALUES (?1, ?2, ?3, ?4, ?5)
              ON CONFLICT(product_type, exchange, exchange_symbol) 
              DO UPDATE SET
                tick_size = excluded.tick_size,
                lot_size = excluded.lot_size,
                updated_at = CURRENT_TIMESTAMP",
            params![
                &item.product_type,
                &item.exchange,
                &item.exchange_symbol,
                &item.tick_size,
                &item.lot_size,
            ],
        )?;
        println!("Saved: {} {} {}", item.exchange, item.product_type, item.exchange_symbol);
    }

    Ok(())
}
