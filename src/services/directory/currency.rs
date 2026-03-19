use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::services::directory::money::{Currency, Money};

const FRANKFURTER_BASE_URL: &str = "https://api.frankfurter.app";
/// Cache exchange rates for 6 hours — ECB updates once daily, no need to hit
/// the API on every salary comparison
const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Clone)]
struct CachedRates {
    /// rates[currency] = how many of that currency equal 1 USD
    rates: HashMap<String, f64>,
    fetched_at: Instant,
}

impl CachedRates {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < CACHE_TTL
    }
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    rates: HashMap<String, f64>,
}

#[derive(Clone)]
pub struct CurrencyConverter {
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedRates>>>,
}

impl CurrencyConverter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to build reqwest client"),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetch rates from Frankfurter with USD as the base.
    /// Results are cached for CACHE_TTL — concurrent callers share one fetch.
    async fn fetch_rates(&self) -> anyhow::Result<HashMap<String, f64>> {
        // Fast path: return cached rates if still fresh
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.is_fresh() {
                    return Ok(cached.rates.clone());
                }
            }
        }

        // Slow path: fetch fresh rates
        let url = format!("{}/latest?base=USD", FRANKFURTER_BASE_URL);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Frankfurter API request failed")?
            .error_for_status()
            .context("Frankfurter returned non-2xx status")?
            .json::<FrankfurterResponse>()
            .await
            .context("Failed to parse Frankfurter response")?;

        // Frankfurter uses USD as base but doesn't include USD → USD in the map.
        // Insert it explicitly so all lookups work uniformly.
        let mut rates = resp.rates;
        rates.insert("USD".to_string(), 1.0);

        let cached = CachedRates {
            rates: rates.clone(),
            fetched_at: Instant::now(),
        };

        *self.cache.write().await = Some(cached);

        Ok(rates)
    }

    /// Convert `money` to USD.
    ///
    /// Returns the original value unchanged if it's already USD.
    /// Returns an error if the currency is not in the Frankfurter rate table.
    pub async fn to_usd(&self, money: Money) -> anyhow::Result<Money> {
        if money.currency == Currency::Usd {
            return Ok(money);
        }

        let rates = self.fetch_rates().await?;
        let code = money.currency.iso_code();

        // rates[code] = units of `code` per 1 USD
        // so: usd_amount = foreign_amount / rates[code]
        let rate = rates
            .get(code)
            .ok_or_else(|| anyhow!("No exchange rate available for {}", code))?;

        Ok(Money::usd(money.amount / rate))
    }

    /// Convert `money` to any target currency.
    pub async fn convert(&self, money: Money, target: Currency) -> anyhow::Result<Money> {
        if money.currency == target {
            return Ok(money);
        }
        // Convert to USD first, then to target
        let usd = self.to_usd(money).await?;
        if target == Currency::Usd {
            return Ok(usd);
        }

        let rates = self.fetch_rates().await?;
        let code = target.iso_code();
        let rate = rates
            .get(code)
            .ok_or_else(|| anyhow!("No exchange rate available for {}", code))?;

        Ok(Money::new(usd.amount * rate, target))
    }
}
