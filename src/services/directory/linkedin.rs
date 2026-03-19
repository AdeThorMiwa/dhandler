use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use di::injectable;
use sea_orm::DatabaseConnection;
use std::{sync::Arc, time::Duration};
use thirtyfour::{prelude::*, ChromeCapabilities};

use crate::{
    models::{
        _entities::sea_orm_active_enums::Modality,
        linkedin_creds::{LinkedinCred, LinkedinCreds},
        linkedin_seen_jobs::LinkedinSeenJobs,
    },
    services::directory::{
        apply::{Answer, Question, QuestionHandler, QuestionKind},
        currency::CurrencyConverter,
        money::{Currency, Money},
        ApplyRequest, ApplyResult, FetchJobsRequest, JobDirectory, JobDirectoryMetadata, JobEntry,
        JobSearchFilters,
    },
    utils::settings::Settings,
};

#[injectable(JobDirectory)]
pub struct LinkedInJobDirectory {
    settings: Arc<Settings>,
    db: Arc<DatabaseConnection>,
    currency_converter: Arc<CurrencyConverter>,
}

#[derive(Debug)]
struct RawListing {
    linkedin_job_id: String,
    title: String,
    company: String,
    location: String,
    job_url: String,
}

struct JobDetail {
    description: String,
    seniority_level: Option<String>,
    employment_type: Option<String>,
    industry: Option<String>,
    job_function: Option<String>,
    applicant_count: Option<String>,
    posted_at: Option<String>,
    // authenticated-only
    salary_range: Option<String>,
    easy_apply: bool,
    recruiter_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LinkedInAuthCredentials {
    pub li_at: String,
    pub jsessionid: Option<String>,
}

fn parse_salary(raw: &str) -> Option<Money> {
    if raw.trim().is_empty() {
        return None;
    }

    // ── 1. Detect currency before we strip symbols ────────────────────────────
    let currency = Currency::detect(raw);

    // ── 2. Detect pay period ──────────────────────────────────────────────────
    let lower = raw.to_lowercase();
    let is_hourly = lower.contains("/hr")
        || lower.contains("/ hr")
        || lower.contains("per hour")
        || lower.contains("an hour");
    let is_monthly = lower.contains("/mo")
        || lower.contains("/ mo")
        || lower.contains("per month")
        || lower.contains("a month");

    // ── 3. Strip everything except digits, '.', '-', 'k', 'K' ────────────────
    // We keep '-' as range separator and 'k'/'K' as thousands shorthand.
    let stripped: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == 'k' || *c == 'K')
        .collect();

    if stripped.is_empty() {
        return None;
    }

    // ── 4. Split on '-' to handle ranges ─────────────────────────────────────
    // Guard against negative numbers (unlikely in salary strings but safe to handle)
    let parts: Vec<f64> = stripped
        .split('-')
        .filter(|s| !s.is_empty())
        .filter_map(|part| {
            let lower_part = part.to_lowercase();
            let has_k = lower_part.ends_with('k');
            let numeric_str = lower_part.trim_end_matches('k');
            numeric_str
                .parse::<f64>()
                .ok()
                .map(|v| if has_k { v * 1_000.0 } else { v })
        })
        .collect();

    let raw_amount = match parts.as_slice() {
        [] => return None,
        [single] => *single,
        [low, high] => (low + high) / 2.0,
        // More than 2 parts — take the outer bounds as low/high
        many => (many[0] + many[many.len() - 1]) / 2.0,
    };

    if raw_amount <= 0.0 {
        return None;
    }

    // ── 5. Annualise ──────────────────────────────────────────────────────────
    let annual = if is_hourly {
        raw_amount * 2_080.0 // 40 hrs × 52 weeks
    } else if is_monthly {
        raw_amount * 12.0
    } else {
        raw_amount
    };

    Some(Money::new(annual, currency))
}

fn search_url(role: &str, start: usize) -> String {
    let encoded = urlencoding::encode(role);
    format!(
        "https://www.linkedin.com/jobs/search?keywords={}&start={}&position=1&pageNum=0",
        encoded, start
    )
}

fn extract_job_id(url: &str) -> Option<String> {
    url.split("/jobs/view/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Safely get trimmed inner text from an element, returning None on any failure.
async fn text_of(el: &WebElement) -> Option<String> {
    el.text()
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn detect_modality(location: &str, description: &str) -> Modality {
    let haystack = format!("{} {}", location, description).to_lowercase();

    if haystack.contains("remote") {
        Modality::Remote
    } else if haystack.contains("hybrid") {
        Modality::Hybrid
    } else {
        Modality::Onsite
    }
}

/// Returns `true` if the job passes all active filters.
async fn passes_filters(
    job: &JobEntry,
    filters: &JobSearchFilters,
    converter: &CurrencyConverter,
) -> bool {
    // ── Org blacklist ─────────────────────────────────────────────────────────
    let company_lower = job.company.to_lowercase();
    if filters
        .org_blacklist
        .iter()
        .any(|blocked| company_lower.contains(&blocked.to_lowercase()))
    {
        tracing::debug!(
            job_id = %job.id,
            company = %job.company,
            "Filtered out: org blacklist"
        );
        return false;
    }

    if let Some(min_usd) = filters.minimum_salary {
        if let Some(raw) = &job.salary_range {
            if let Some(money) = parse_salary(raw) {
                match converter.to_usd(money).await {
                    Ok(usd_money) if usd_money.amount < min_usd => {
                        tracing::debug!(
                            job_id = %job.id,
                            salary = %money,
                            usd_equivalent = usd_money.amount,
                            minimum_usd = min_usd,
                            "Filtered: below minimum salary"
                        );
                        return false;
                    }
                    Err(e) => {
                        // Conversion failed (network, unknown currency) — let job through
                        tracing::warn!(
                            job_id = %job.id,
                            error = %e,
                            "Currency conversion failed, allowing job through"
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Modality ──────────────────────────────────────────────────────────────
    if let Some(allowed) = &filters.modalities {
        if !allowed.is_empty() {
            let job_modality = detect_modality(&job.location, &job.description);
            if !allowed.contains(&job_modality) {
                tracing::debug!(
                    job_id = %job.id,
                    modality = ?job_modality,
                    "Filtered out: modality not in allowed list"
                );
                return false;
            }
        }
    }

    // ── Country ───────────────────────────────────────────────────────────────
    if let Some(country) = &filters.country {
        if !job
            .location
            .to_lowercase()
            .contains(&country.to_lowercase())
        {
            tracing::debug!(
                job_id = %job.id,
                location = %job.location,
                country = %country,
                "Filtered out: country mismatch"
            );
            return false;
        }
    }

    true
}

impl LinkedInJobDirectory {
    async fn fetch_jobs_with_driver(
        &self,
        driver: &WebDriver,
        request: FetchJobsRequest,
        creds: &LinkedinCred,
    ) -> anyhow::Result<Vec<JobEntry>> {
        let FetchJobsRequest {
            role,
            ref_id,
            limit,
            filters,
        } = request;

        self.authenticate_with(driver, creds).await?;

        let seen_jobs = LinkedinSeenJobs::find_by_ref_id_and_role(&self.db, &ref_id, &role).await?;

        let mut fresh: Vec<JobEntry> = Vec::new();
        let mut newly_seen_ids: Vec<String> = Vec::new();

        let page_size = self.settings.linkedin.pagination_size;
        let max_pages = self.settings.linkedin.pagination_max_pages;

        'outer: for page in 0..max_pages {
            let listings = self
                .scrape_search_page(driver, &role, page * page_size)
                .await
                .context(format!("Failed scraping page {page}"))?;

            if listings.is_empty() {
                break;
            }

            for raw in listings {
                // Already seen by this user for this role — skip without recording
                if seen_jobs.contains(&raw.linkedin_job_id)
                    || newly_seen_ids.contains(&raw.linkedin_job_id)
                {
                    continue;
                }

                let detail =
                    self.scrape_job_detail(driver, &raw.job_url)
                        .await
                        .context(format!(
                            "Failed fetching detail for {}",
                            raw.linkedin_job_id
                        ))?;

                let modality = detect_modality(&raw.location, &detail.description);

                let entry = JobEntry {
                    id: raw.linkedin_job_id.clone(),
                    title: raw.title,
                    company: raw.company,
                    location: raw.location,
                    url: raw.job_url,
                    description: detail.description,
                    seniority_level: detail.seniority_level,
                    employment_type: detail.employment_type,
                    industry: detail.industry,
                    job_function: detail.job_function,
                    applicant_count: detail.applicant_count,
                    posted_at: detail.posted_at,
                    salary_range: detail.salary_range,
                    easy_apply: detail.easy_apply,
                    recruiter_name: detail.recruiter_name,
                    modality,
                    source: "linkedin".to_string(),
                };

                // ── Filter gate ───────────────────────────────────────────────
                // Filtered jobs are NOT recorded as seen — they can be
                // re-evaluated on the next fetch if filters change.
                if !passes_filters(&entry, &filters, &self.currency_converter).await {
                    continue;
                }

                // Passed all filters — mark as seen and collect
                newly_seen_ids.push(raw.linkedin_job_id);
                fresh.push(entry);

                if fresh.len() == limit {
                    break 'outer;
                }
            }
        }

        if fresh.is_empty() {
            return Err(anyhow!(
                "No unseen LinkedIn jobs matching the given filters for role '{}' \
                         (ref_id: '{}'). All available results have been surfaced or filtered out.",
                role,
                ref_id
            ));
        }

        for seen in newly_seen_ids {
            LinkedinSeenJobs::mark_seen(&self.db, &ref_id, &role, &seen).await?;
        }

        Ok(fresh)
    }

    async fn scrape_search_page(
        &self,
        driver: &WebDriver,
        role: &str,
        start: usize,
    ) -> anyhow::Result<Vec<RawListing>> {
        driver.goto(&search_url(role, start)).await?;
        tokio::time::sleep(Duration::from_millis(2_500)).await;

        // Scroll to trigger lazy-loaded cards
        driver
            .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
            .await?;
        tokio::time::sleep(Duration::from_millis(1_000)).await;

        let cards = driver
            .find_all(By::Css("ul.jobs-search__results-list > li"))
            .await
            .context("Job card list not found — LinkedIn markup may have changed")?;

        let mut listings = Vec::new();

        for card in &cards {
            let anchor = match card.find(By::Css("a.base-card__full-link")).await {
                Ok(el) => el,
                Err(_) => continue,
            };

            let job_url = match anchor.attr("href").await? {
                Some(u) if !u.is_empty() => u.split('?').next().unwrap_or(&u).to_string(),
                _ => continue,
            };

            let Some(linkedin_job_id) = extract_job_id(&job_url) else {
                continue;
            };

            let title = match card.find(By::Css("h3.base-search-card__title")).await {
                Ok(el) => text_of(&el).await.unwrap_or_default(),
                Err(_) => continue, // title is mandatory
            };

            let company = card
                .find(By::Css("h4.base-search-card__subtitle"))
                .await
                .ok()
                .and_then(|el| {
                    let t = text_of(&el);
                    // block_on is fine here — we're inside an async fn driving
                    // thirtyfour which uses its own internal executor per element op
                    futures::executor::block_on(t)
                })
                .unwrap_or_default();

            let location = card
                .find(By::Css("span.job-search-card__location"))
                .await
                .ok()
                .and_then(|el| futures::executor::block_on(text_of(&el)))
                .unwrap_or_default();

            listings.push(RawListing {
                linkedin_job_id,
                title,
                company,
                location,
                job_url,
            });
        }

        Ok(listings)
    }

    async fn scrape_job_detail(
        &self,
        driver: &WebDriver,
        job_url: &str,
    ) -> anyhow::Result<JobDetail> {
        driver.goto(job_url).await?;
        tokio::time::sleep(Duration::from_millis(2_000)).await;

        // Expand truncated description
        if let Ok(btn) = driver
            .find(By::Css("button.show-more-less-html__button--more"))
            .await
        {
            let _ = btn.click().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let description = driver
            .find(By::Css("div.show-more-less-html__markup"))
            .await
            .ok()
            .and_then(|el| futures::executor::block_on(text_of(&el)))
            .unwrap_or_default();

        // ── Criteria items (seniority, type, industry, function) ─────────────
        let mut seniority_level = None;
        let mut employment_type = None;
        let mut industry = None;
        let mut job_function = None;

        for item in driver
            .find_all(By::Css("li.description__job-criteria-item"))
            .await
            .unwrap_or_default()
        {
            let header = item
                .find(By::Css("h3.description__job-criteria-subheader"))
                .await
                .ok()
                .and_then(|el| futures::executor::block_on(text_of(&el)))
                .unwrap_or_default()
                .to_lowercase();

            let value = item
                .find(By::Css("span.description__job-criteria-text"))
                .await
                .ok()
                .and_then(|el| futures::executor::block_on(text_of(&el)));

            match header.trim() {
                "seniority level" => seniority_level = value,
                "employment type" => employment_type = value,
                "industries" => industry = value,
                "job function" => job_function = value,
                _ => {}
            }
        }

        let applicant_count = driver
            .find(By::Css(
                "figcaption.num-applicants__caption, \
                     span.num-applicants__caption, \
                     span.jobs-unified-top-card__applicant-count",
            ))
            .await
            .ok()
            .and_then(|el| futures::executor::block_on(text_of(&el)));

        let posted_at = driver
            .find(By::Css(
                "span.posted-time-ago__text, \
                     span.jobs-unified-top-card__posted-date, \
                     time",
            ))
            .await
            .ok()
            .and_then(|el| futures::executor::block_on(text_of(&el)));

        // ── Authenticated-only fields ─────────────────────────────────────────
        let salary_range = {
            // LinkedIn renders salary in a few different elements depending on
            // whether it's employer-provided or estimated
            driver
                .find(By::Css(
                    "div.salary.compensation__salary, \
                         span.jobs-unified-top-card__job-insight--highlight, \
                         li.jobs-unified-top-card__job-insight span[aria-hidden='false']",
                ))
                .await
                .ok()
                .and_then(|el| futures::executor::block_on(text_of(&el)))
                .filter(|s| {
                    s.contains('$')
                        || s.to_lowercase().contains("yr")
                        || s.contains('€')
                        || s.contains('£')
                })
        };

        // Easy Apply button only exists on authenticated sessions
        let easy_apply = driver
            .find(By::Css("button.jobs-apply-button--top-card"))
            .await
            .ok()
            .and_then(|el| futures::executor::block_on(el.text()).ok())
            .map(|t| t.to_lowercase().contains("easy apply"))
            .unwrap_or(false);

        // Recruiter / hiring manager card — only shown to logged-in users
        let recruiter_name = driver
            .find(By::Css(
                "a.hirer-card__hirer-information span.jobs-poster__name, \
                         span.recruitment-agency-card__recruiter-name",
            ))
            .await
            .ok()
            .and_then(|el| futures::executor::block_on(text_of(&el)));

        Ok(JobDetail {
            description,
            seniority_level,
            employment_type,
            industry,
            job_function,
            applicant_count,
            posted_at,
            salary_range,
            easy_apply,
            recruiter_name,
        })
    }

    fn build_driver_caps() -> anyhow::Result<ChromeCapabilities> {
        let mut caps = DesiredCapabilities::chrome();

        let args = [
            // Use the new headless mode — less detectable than --headless=old
            "--headless=new",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            // Core anti-detection flags
            "--disable-blink-features=AutomationControlled",
            "--disable-infobars",
            "--disable-extensions",
            // Make the window size realistic — headless default is tiny and detectable
            "--window-size=1920,1080",
            "--start-maximized",
            // Disable automation-related Chrome features
            "--disable-automation",
            // Prevent navigator.webdriver from being true
            "--exclude-switches=enable-automation",
            // GPU flags — absence of GPU is a headless tell
            "--disable-gpu",
            "--disable-software-rasterizer",
            // Language + locale — blank locale is a headless tell
            "--lang=en-US,en",
            "--accept-lang=en-US,en;q=0.9",
        ];

        for arg in args {
            caps.add_arg(arg)?;
        }

        // Realistic user agent — must match the Chrome version your chromedriver runs
        caps.add_arg(
            "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/124.0.0.0 Safari/537.36",
        )?;

        // Disable the `navigator.webdriver` property via Chrome preferences
        caps.add_experimental_option("excludeSwitches", serde_json::json!(["enable-automation"]))?;
        caps.add_experimental_option("useAutomationExtension", serde_json::json!(false))?;

        Ok(caps)
    }

    /// Inject resolved credentials into the driver session.
    async fn authenticate_with(
        &self,
        driver: &WebDriver,
        creds: &LinkedinCred,
    ) -> anyhow::Result<()> {
        // Navigate to a static endpoint to establish domain context without
        // triggering auth-wall redirects
        driver.goto("https://www.linkedin.com/robots.txt").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Inject li_at
        let mut li_at = Cookie::new("li_at", &creds.li_at);
        li_at.set_domain(".linkedin.com");
        li_at.set_path("/");
        li_at.set_secure(true);
        li_at.set_same_site(SameSite::None);
        driver.add_cookie(li_at).await?;

        // Inject JSESSIONID if provided
        if let Some(jsid) = &creds.j_session_id {
            let mut jc = Cookie::new("JSESSIONID", format!("\"{}\"", jsid));
            jc.set_domain(".linkedin.com");
            jc.set_path("/");
            jc.set_secure(true);
            jc.set_same_site(SameSite::None);
            driver.add_cookie(jc).await?;
        }

        // Now navigate to feed with cookies already in the jar
        driver.goto("https://www.linkedin.com/feed/").await?;
        tokio::time::sleep(Duration::from_millis(3_000)).await;

        // Verify we landed somewhere authenticated
        let url = driver.current_url().await?.to_string();
        if url.contains("authwall")
            || url.contains("/login")
            || url.contains("checkpoint")
            || url.contains("redirect")
        {
            return Err(anyhow!(
                "Authentication failed — still on '{}' after cookie injection. \
                         The li_at cookie may have expired. \
                         Re-capture it from DevTools → Application → Cookies → li_at \
                         while logged into LinkedIn.",
                url
            ));
        }

        tracing::info!("LinkedIn: authenticated successfully, landed on {}", url);
        Ok(())
    }

    async fn run_easy_apply(
        &self,
        driver: &WebDriver,
        job: JobEntry,
        request: ApplyRequest,
        creds: LinkedinCred,
    ) -> anyhow::Result<ApplyResult> {
        // ── 1. Authenticate with resolved per-user credentials ─────────────
        self.authenticate_with(driver, &creds).await?;

        // ── 2. Navigate to job listing ─────────────────────────────────────
        driver.goto(&job.url).await?;
        tokio::time::sleep(Duration::from_millis(2_000)).await;

        // ── 3. Click Easy Apply ────────────────────────────────────────────
        let apply_btn = driver
            .find(By::Css("button.jobs-apply-button--top-card"))
            .await
            .context("Easy Apply button not found — job may no longer be active")?;

        apply_btn.click().await?;
        tokio::time::sleep(Duration::from_millis(1_500)).await;

        // ── 4. Walk modal steps ────────────────────────────────────────────
        let max_steps = 15usize;

        for step in 0..max_steps {
            tracing::debug!(step, job_id = %job.id, "Easy Apply: processing step");

            let action_btn = match self.find_modal_action_button(driver).await {
                Some(btn) => btn,
                None => {
                    let screenshot = self.take_screenshot(driver).await;
                    return Ok(ApplyResult::RequiresManualAction {
                        job_id: job.id.clone(),
                        reason: format!("Step {step}: could not find Next/Review/Submit button"),
                        screenshot,
                    });
                }
            };

            let btn_text = action_btn.text().await?.to_lowercase();

            // Fill every visible question on this step via the handler
            let fill_result = self
                .fill_modal_step_with_handler(driver, &*request.question_handler, step)
                .await;

            if let Err(e) = fill_result {
                let screenshot = self.take_screenshot(driver).await;
                return Ok(ApplyResult::RequiresManualAction {
                    job_id: job.id.clone(),
                    reason: format!("Step {step}: {e}"),
                    screenshot,
                });
            }

            if btn_text.contains("submit") {
                action_btn.click().await?;
                tokio::time::sleep(Duration::from_millis(2_000)).await;

                let confirmed = driver
                    .find(By::Css(
                        "div.jobs-easy-apply-modal--success, \
                             h2[data-test-modal-close-btn], \
                             div.artdeco-modal__dismiss",
                    ))
                    .await
                    .is_ok();

                return if confirmed {
                    Ok(ApplyResult::Applied {
                        job_id: job.id.clone(),
                        applied_at: chrono::Utc::now(),
                    })
                } else {
                    let screenshot = self.take_screenshot(driver).await;
                    Ok(ApplyResult::RequiresManualAction {
                        job_id: job.id.clone(),
                        reason: "Submit clicked but no confirmation modal appeared. \
                                     Application status uncertain."
                            .to_string(),
                        screenshot,
                    })
                };
            }

            if btn_text.contains("next")
                || btn_text.contains("review")
                || btn_text.contains("continue")
            {
                action_btn.click().await?;
                tokio::time::sleep(Duration::from_millis(1_200)).await;
                continue;
            }

            let screenshot = self.take_screenshot(driver).await;
            return Ok(ApplyResult::RequiresManualAction {
                job_id: job.id.clone(),
                reason: format!("Step {step}: unrecognised modal action '{btn_text}'"),
                screenshot,
            });
        }

        let screenshot = self.take_screenshot(driver).await;
        Ok(ApplyResult::RequiresManualAction {
            job_id: job.id.clone(),
            reason: format!("Modal exceeded {max_steps} steps without reaching Submit."),
            screenshot,
        })
    }

    /// Find the primary action button (Next / Review / Submit) in the modal.
    async fn find_modal_action_button(&self, driver: &WebDriver) -> Option<WebElement> {
        // LinkedIn uses a few different selectors across modal versions
        let selectors = [
            "button[aria-label='Submit application']",
            "button[aria-label='Continue to next step']",
            "button[aria-label='Review your application']",
            "footer.jobs-easy-apply-modal__footer button.artdeco-button--primary",
            "div.jobs-easy-apply-modal__footer button.artdeco-button--primary",
        ];

        for selector in selectors {
            if let Ok(btn) = driver.find(By::Css(selector)).await {
                return Some(btn);
            }
        }
        None
    }

    /// Try to find the <label> associated with a form element.
    /// Falls back to the element's placeholder or aria-label attribute.
    async fn label_for_input(&self, driver: &WebDriver, el: &WebElement) -> String {
        // Try `for` attribute → matching <label>
        if let Ok(Some(id)) = el.attr("id").await {
            if !id.is_empty() {
                if let Ok(label) = driver.find(By::Css(&format!("label[for='{}']", id))).await {
                    if let Ok(text) = label.text().await {
                        if !text.is_empty() {
                            return text;
                        }
                    }
                }
            }
        }

        // Try aria-label on the element itself
        if let Ok(Some(aria)) = el.attr("aria-label").await {
            if !aria.is_empty() {
                return aria;
            }
        }

        // Try placeholder
        if let Ok(Some(ph)) = el.attr("placeholder").await {
            if !ph.is_empty() {
                return ph;
            }
        }

        String::new()
    }

    /// Capture a base64-encoded PNG screenshot for RequiresManualAction results.
    async fn take_screenshot(&self, driver: &WebDriver) -> Option<String> {
        match driver.screenshot_as_png().await {
            Ok(bytes) => Some(general_purpose::STANDARD.encode(bytes)),
            Err(e) => {
                tracing::warn!("Failed to take screenshot: {}", e);
                None
            }
        }
    }

    async fn fill_modal_step_with_handler(
        &self,
        driver: &WebDriver,
        handler: &dyn QuestionHandler,
        step: usize,
    ) -> anyhow::Result<()> {
        // ── File upload (resume) — step 0 only ────────────────────────────
        if step == 0 {
            if let Ok(input) = driver
                .find(By::Css("input[type='file'][accept='.pdf,.doc,.docx']"))
                .await
            {
                let question = Question::new(
                    "Upload your resume",
                    QuestionKind::FileUpload,
                    true,
                    vec![],
                    Some("PDF, DOC, or DOCX".to_string()),
                    None,
                );

                match handler.answer(&question).await? {
                    Answer::FileUpload { filename, bytes } => {
                        let tmp = std::env::temp_dir().join(&filename);
                        tokio::fs::write(&tmp, &bytes).await?;
                        input.send_keys(tmp.to_string_lossy().as_ref()).await?;
                        tokio::time::sleep(Duration::from_millis(1_500)).await;
                        let _ = tokio::fs::remove_file(&tmp).await;
                    }
                    Answer::Skip => {}
                    other => {
                        tracing::warn!(?other, "Unexpected answer type for file upload — skipping");
                    }
                }
            }
        }

        // ── Text / number inputs ──────────────────────────────────────────
        let inputs = driver
            .find_all(By::Css(
                "div.jobs-easy-apply-modal \
                     input[type='text']:not([disabled]), \
                     div.jobs-easy-apply-modal \
                     input[type='email']:not([disabled]), \
                     div.jobs-easy-apply-modal \
                     input[type='tel']:not([disabled]), \
                     div.jobs-easy-apply-modal \
                     input[type='number']:not([disabled])",
            ))
            .await
            .unwrap_or_default();

        for input in &inputs {
            let kind = match input.attr("type").await?.as_deref() {
                Some("number") => QuestionKind::Number,
                _ => QuestionKind::Text,
            };
            let required = input.attr("required").await?.is_some();
            let hint = input.attr("placeholder").await?.filter(|s| !s.is_empty());
            let current = input.value().await?.filter(|s| !s.is_empty());
            let label = self.label_for_input(driver, input).await;

            // Skip fields that are already filled and not required to change
            if current.is_some() && !required {
                continue;
            }

            let question = Question::new(label, kind, required, vec![], hint, current);

            match handler.answer(&question).await? {
                Answer::Text(val) => {
                    input.clear().await?;
                    input.send_keys(&val).await?;
                }
                Answer::Skip => {}
                other => {
                    tracing::warn!(?other, "Unexpected answer type for text input");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // ── Textareas ─────────────────────────────────────────────────────
        let textareas = driver
            .find_all(By::Css(
                "div.jobs-easy-apply-modal textarea:not([disabled])",
            ))
            .await
            .unwrap_or_default();

        for textarea in &textareas {
            let required = textarea.attr("required").await?.is_some();
            let hint = textarea
                .attr("placeholder")
                .await?
                .filter(|s| !s.is_empty());
            let current = textarea.value().await?.filter(|s| !s.is_empty());
            let label = self.label_for_input(driver, textarea).await;

            let question =
                Question::new(label, QuestionKind::Text, required, vec![], hint, current);

            match handler.answer(&question).await? {
                Answer::Text(val) => {
                    textarea.clear().await?;
                    textarea.send_keys(&val).await?;
                }
                Answer::Skip => {}
                other => {
                    tracing::warn!(?other, "Unexpected answer type for textarea");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // ── Dropdowns (<select>) ──────────────────────────────────────────
        let selects = driver
            .find_all(By::Css("div.jobs-easy-apply-modal select:not([disabled])"))
            .await
            .unwrap_or_default();

        for select in &selects {
            let required = select.attr("required").await?.is_some();
            let label = self.label_for_input(driver, select).await;
            let options: Vec<String> = select
                .find_all(By::Css("option"))
                .await
                .unwrap_or_default()
                .iter()
                .filter_map(|o| futures::executor::block_on(o.text()).ok())
                .filter(|t| !t.is_empty())
                .collect();

            let current = select
                .find_all(By::Css("option[selected]"))
                .await
                .unwrap_or_default()
                .first()
                .and_then(|o| futures::executor::block_on(o.text()).ok());

            let question = Question::new(
                label,
                QuestionKind::Dropdown,
                required,
                options.clone(),
                None,
                current,
            );

            match handler.answer(&question).await? {
                Answer::SingleChoice(chosen) | Answer::Text(chosen) => {
                    for option in select.find_all(By::Css("option")).await.unwrap_or_default() {
                        let text = option.text().await.unwrap_or_default();
                        if text.to_lowercase().contains(&chosen.to_lowercase()) {
                            option.click().await?;
                            break;
                        }
                    }
                }
                Answer::Skip => {}
                other => {
                    tracing::warn!(?other, "Unexpected answer type for select");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // ── Fieldsets (radio + checkbox groups) ───────────────────────────
        let fieldsets = driver
            .find_all(By::Css("div.jobs-easy-apply-modal fieldset"))
            .await
            .unwrap_or_default();

        for fieldset in &fieldsets {
            let legend = fieldset
                .find(By::Css("legend span"))
                .await
                .ok()
                .and_then(|el| futures::executor::block_on(el.text()).ok())
                .unwrap_or_default();

            let required = fieldset.attr("required").await?.is_some();

            // Detect whether this fieldset is checkboxes or radios
            let is_multi = fieldset
                .find_all(By::Css("input[type='checkbox']"))
                .await
                .map(|v| !v.is_empty())
                .unwrap_or(false);

            let options: Vec<String> = fieldset
                .find_all(By::Css("label"))
                .await
                .unwrap_or_default()
                .iter()
                .filter_map(|l| futures::executor::block_on(l.text()).ok())
                .filter(|t| !t.is_empty())
                .collect();

            // Detect yes/no
            let kind = if !is_multi
                && options.len() == 2
                && options.iter().any(|o| o.to_lowercase() == "yes")
                && options.iter().any(|o| o.to_lowercase() == "no")
            {
                QuestionKind::YesNo
            } else if is_multi {
                QuestionKind::MultiChoice
            } else {
                QuestionKind::SingleChoice
            };

            let question = Question::new(legend, kind.clone(), required, options, None, None);

            match handler.answer(&question).await? {
                Answer::YesNo(yes) => {
                    let target = if yes { "yes" } else { "no" };
                    for radio in fieldset
                        .find_all(By::Css("input[type='radio']"))
                        .await
                        .unwrap_or_default()
                    {
                        let lbl = self.label_for_input(driver, &radio).await.to_lowercase();
                        if lbl.contains(target) {
                            radio.click().await?;
                            break;
                        }
                    }
                }
                Answer::SingleChoice(chosen) => {
                    for radio in fieldset
                        .find_all(By::Css("input[type='radio']"))
                        .await
                        .unwrap_or_default()
                    {
                        let lbl = self.label_for_input(driver, &radio).await.to_lowercase();
                        if lbl.contains(&chosen.to_lowercase()) {
                            radio.click().await?;
                            break;
                        }
                    }
                }
                Answer::MultiChoice(chosen_list) => {
                    for checkbox in fieldset
                        .find_all(By::Css("input[type='checkbox']"))
                        .await
                        .unwrap_or_default()
                    {
                        let lbl = self.label_for_input(driver, &checkbox).await.to_lowercase();
                        if chosen_list.iter().any(|c| lbl.contains(&c.to_lowercase())) {
                            checkbox.click().await?;
                        }
                    }
                }
                Answer::Skip => {}
                other => {
                    tracing::warn!(?other, "Unexpected answer for fieldset kind {kind:?}");
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

#[async_trait]
impl JobDirectory for LinkedInJobDirectory {
    fn metadata(&self) -> JobDirectoryMetadata {
        JobDirectoryMetadata {
            id: "linkedin".to_string(),
            name: "LinkedIn".to_string(),
        }
    }

    async fn fetch_jobs(&self, req: FetchJobsRequest) -> anyhow::Result<Vec<JobEntry>> {
        println!("inside");
        let creds = LinkedinCreds::get_by_ref_id(&self.db, &req.ref_id).await?;

        println!("after");

        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--no-sandbox")?;
        caps.add_arg("--disable-dev-shm-usage")?;
        caps.add_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_arg(
            "--user-agent=Mozilla/5.0 (X11; Linux x86_64) \
                     AppleWebKit/537.36 (KHTML, like Gecko) \
                     Chrome/124.0.0.0 Safari/537.36",
        )?;

        let driver = WebDriver::new(&self.settings.linkedin.webdriver_url, caps).await?;
        let result = self.fetch_jobs_with_driver(&driver, req, &creds).await;
        // let _ = driver.quit().await;

        tokio::time::sleep(Duration::from_millis(500000)).await;
        result
    }

    async fn apply(&self, job: JobEntry, request: ApplyRequest) -> anyhow::Result<ApplyResult> {
        if !job.easy_apply {
            // NOTE: external apply not yet implemented
            return Ok(ApplyResult::ExternalApplicationRequired {
                job_id: job.id.clone(),
                external_url: job.url.clone(),
            });
        }

        // Resolve credentials before touching the browser
        let creds = LinkedinCreds::get_by_ref_id(&self.db, &request.ref_id).await?;

        let caps = Self::build_driver_caps()?;
        let driver = WebDriver::new(&self.settings.linkedin.webdriver_url, caps).await?;
        let result = self.run_easy_apply(&driver, job, request, creds).await;
        let _ = driver.quit().await;
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::App, utils::app::create_di_provider};
    use di::ServiceProvider;
    use loco_rs::{app::Hooks, boot::create_context, environment::Environment};
    use uuid::Uuid;

    use super::*;

    async fn setup() -> std::io::Result<ServiceProvider> {
        let env = Environment::Development;
        let config = App::load_config(&env).await.expect("failed to load config");
        let ctx = create_context::<App>(&env, config)
            .await
            .expect("failed to create context");

        Ok(create_di_provider(&ctx))
    }

    async fn seed_creds(provider: &ServiceProvider, ref_id: &str) {
        let db = provider.get_required::<DatabaseConnection>();
        let settings = provider.get_required::<Settings>();

        LinkedinCreds::upsert(
            &db,
            ref_id,
            LinkedInAuthCredentials {
                li_at: settings.linkedin.li_at.clone().unwrap(),
                jsessionid: settings.linkedin.jsessionid.clone(),
            },
        )
        .await
        .expect("failed to seed creds")
    }

    #[tokio::test]
    async fn test_fetch_jobs() {
        let provider = setup().await.expect("failed to init provider");
        let directory = provider.get_required::<dyn JobDirectory>();

        let ref_id = Uuid::new_v4();

        seed_creds(&provider, &ref_id.to_string()).await;

        println!("ref id: {ref_id} directory: {:?}", directory.metadata());

        let request = FetchJobsRequest {
            role: "software engineer".to_string(),
            ref_id: ref_id.to_string(),
            limit: 2,
            filters: JobSearchFilters::default(),
        };

        let response = directory
            .fetch_jobs(request)
            .await
            .expect("fetch_jobs call failed");

        println!("hahahaha");

        assert_eq!(response.len(), 2, "expected 2 jobs, got {}", response.len());

        println!("jobs: {:?}", response);
    }
}
