use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Usd,
    Eur,
    Gbp,
    Cad,
    Aud,
    Chf,
    Jpy,
    Inr,
    Brl,
    Sgd,
    Hkd,
    Nzd,
    Mxn,
    Sek,
    Nok,
    Dkk,
    Zar,
    Aed,
}

impl Currency {
    /// The symbol(s) that uniquely identify this currency in a raw salary string.
    /// Order matters — more specific patterns first.
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::Usd => "$",
            Currency::Eur => "€",
            Currency::Gbp => "£",
            Currency::Jpy => "¥",
            Currency::Inr => "₹",
            Currency::Brl => "R$",
            Currency::Cad => "CA$",
            Currency::Aud => "A$",
            Currency::Sgd => "S$",
            Currency::Hkd => "HK$",
            Currency::Nzd => "NZ$",
            Currency::Mxn => "MX$",
            Currency::Chf => "CHF",
            Currency::Sek => "SEK",
            Currency::Nok => "NOK",
            Currency::Dkk => "DKK",
            Currency::Zar => "ZAR",
            Currency::Aed => "AED",
        }
    }

    /// ISO 4217 code — used in Frankfurter API requests.
    pub fn iso_code(&self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Cad => "CAD",
            Currency::Aud => "AUD",
            Currency::Chf => "CHF",
            Currency::Jpy => "JPY",
            Currency::Inr => "INR",
            Currency::Brl => "BRL",
            Currency::Sgd => "SGD",
            Currency::Hkd => "HKD",
            Currency::Nzd => "NZD",
            Currency::Mxn => "MXN",
            Currency::Sek => "SEK",
            Currency::Nok => "NOK",
            Currency::Dkk => "DKK",
            Currency::Zar => "ZAR",
            Currency::Aed => "AED",
        }
    }

    /// Detect currency from a raw salary string by scanning for known symbols.
    /// More specific multi-char symbols are checked before single-char ones.
    pub fn detect(raw: &str) -> Self {
        // Check multi-char symbols first to avoid e.g. "CA$" matching "$" (USD)
        let candidates = [
            Currency::Cad,
            Currency::Aud,
            Currency::Sgd,
            Currency::Hkd,
            Currency::Nzd,
            Currency::Mxn,
            Currency::Brl,
            Currency::Chf,
            Currency::Sek,
            Currency::Nok,
            Currency::Dkk,
            Currency::Zar,
            Currency::Aed,
            // Single-char symbols last
            Currency::Eur,
            Currency::Gbp,
            Currency::Jpy,
            Currency::Inr,
            Currency::Usd, // plain "$" — fallback
        ];

        for currency in candidates {
            if raw.contains(currency.symbol()) {
                return currency;
            }
        }

        // No symbol found — assume USD (most common on LinkedIn)
        Currency::Usd
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_code())
    }
}

// ── Money ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Money {
    /// Annualised amount in `currency`
    pub amount: f64,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: f64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn usd(amount: f64) -> Self {
        Self::new(amount, Currency::Usd)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.amount, self.currency)
    }
}
