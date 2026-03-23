use crate::services::directory::{
    Answer, ApplyRequest, ApplyResult, ExperienceLevel, FetchJobFilters, FetchJobRequest,
    JobDirectory, JobEntry, Modality, Money, Question, Salary,
};
use chromiumoxide::{Browser, Element, Page};
use chrono::Utc;
use di::injectable;
use loco_rs::prelude::*;
use std::{io, time::Duration};

#[derive(Debug)]
pub struct JobDetails {
    id: String,
    url: String,
    title: String,
    description: String,
    company: String,
    location: String,
    posted_at: String,
    modalities: Vec<Modality>,
    seniority: ExperienceLevel,
    industry: Option<String>,
    salary: Option<Salary>,
    recruiter_name: Option<String>,
}

impl From<JobDetails> for JobEntry {
    fn from(value: JobDetails) -> Self {
        JobEntry {
            id: value.id,
            title: value.title,
            company: value.company.clone(),
            location: value.location,
            url: value.url,
            description: value.description,
            seniority_level: Some(value.seniority),
            industry: value.industry,
            posted_at: Some(value.posted_at),
            salary: value.salary,
            recruiter_name: value.recruiter_name.unwrap_or(value.company),
            modalities: value.modalities,
            source: "https://www.linkedin.com".to_string(),
        }
    }
}

#[injectable(JobDirectory)]
pub struct LinkedinJobDirectory {}

impl LinkedinJobDirectory {
    pub const ID: &'static str = "linkedin";

    fn build_start_url(&self, filters: &FetchJobFilters) -> String {
        // https://www.linkedin.com/jobs/search/?currentJobId=4387471341&f_WT=2%2C3&geoId=103644278&keywords=Softaware%20engineer&origin=JOB_SEARCH_PAGE_JOB_FILTER&refresh=true&spellCorrectionEnabled=true
        // applicable filters
        // f_WT = 1 = Onsite 2 = Remote 3 = Hybrid
        // keyword = Software engineer
        // origin = JOB_SEARCH_PAGE_JOB_FILTER
        // refresh = true
        // f_AL = true if should only show easy apply
        // https://www.linkedin.com/jobs/search/?currentJobId=4384861639&distance=25&f_AL=true&f_C=9206256%2C11111093&f_WT=1%2C3&keywords=Software%20Engineer&location=Lagos%20State%2C%20Nigeria&origin=JOB_SEARCH_PAGE_JOB_FILTER&refresh=true&sortBy=R&spellCorrectionEnabled=true
        let mut base_url = format!("https://www.linkedin.com/jobs/search?keywords={}&origin=JOB_SEARCH_PAGE_JOB_FILTER&refresh=true&spellCorrectionEnabled=true&f_AL=true", filters.role);

        if let Some(modalities) = &filters.modalities {
            let modality_codes = modalities
                .iter()
                .map(Self::resolve_modality)
                .map(|code| code.to_string())
                .collect::<Vec<String>>()
                .join(",");

            base_url = format!("{base_url}&f_WT={}", modality_codes);
        }

        if let Some(location) = &filters.location {
            base_url = format!("{base_url}&location={}", location);
        }

        if let Some(experience) = &filters.experience_level {
            let experience_str = experience
                .iter()
                .map(Self::resolve_experience)
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(",");

            base_url = format!("{base_url}&f_E={}", experience_str);
        }

        base_url
    }

    fn resolve_modality(modality: &Modality) -> u8 {
        match modality {
            Modality::Onsite => 1,
            Modality::Remote => 2,
            Modality::Hybrid => 3,
        }
    }

    fn resolve_experience(experience: &ExperienceLevel) -> u8 {
        match experience {
            ExperienceLevel::Internship => 1,
            ExperienceLevel::EntryLevel => 2,
            ExperienceLevel::Associate => 3,
            ExperienceLevel::MidLevel => 4,
            ExperienceLevel::SeniorLevel => 4,
            ExperienceLevel::Director => 5,
            ExperienceLevel::Executive => 6,
            ExperienceLevel::NotApplicable => 0,
        }
    }

    pub async fn open_jobs_page(
        &self,
        request: &FetchJobRequest,
        browser: &mut Browser,
    ) -> Result<Page> {
        let start_url = self.build_start_url(&request.filters);

        let page = browser
            .new_page(&start_url)
            .await
            .map_err(io::Error::other)?;

        page.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .await
            .map_err(io::Error::other)?;

        page.evaluate_on_new_document(
            r#"
            Object.defineProperty(navigator, "webdriver", { get: () => undefined });
            "#,
        )
        .await
        .map_err(io::Error::other)?;

        tokio::time::sleep(Duration::from_secs(1)).await;

        Ok(page)
    }

    async fn get_text(page: &Page, selector: &str) -> Option<String> {
        page.find_element(selector)
            .await
            .ok()?
            .inner_text()
            .await
            .ok()?
            .map(|s| s.trim().to_string())
    }

    fn parse_modality(s: &str) -> Option<Modality> {
        let lower = s.to_lowercase();
        if lower.contains("remote") {
            Some(Modality::Remote)
        } else if lower.contains("hybrid") {
            Some(Modality::Hybrid)
        } else if lower.contains("on-site")
            || lower.contains("onsite")
            || lower.contains("in-person")
        {
            Some(Modality::Onsite)
        } else {
            None
        }
    }

    async fn scrape_modalities(page: &chromiumoxide::Page) -> Vec<Modality> {
        let mut modalities: Vec<Modality> = Vec::new();

        // Source 1: fit-level preference buttons (most reliable)
        // <div class="job-details-fit-level-preferences"> → <button> → <span>
        if let Ok(buttons) = page
            .find_elements(".job-details-fit-level-preferences button span.tvm__text")
            .await
        {
            for btn in &buttons {
                if let Ok(Some(text)) = btn.inner_text().await {
                    if let Some(m) = Self::parse_modality(text.trim()) {
                        modalities.push(m);
                    }
                }
            }
        }

        // Source 2: fallback — sticky header subtitle contains e.g. "(Hybrid)"
        if modalities.is_empty() {
            if let Ok(elem) = page
                .find_element(".job-details-jobs-unified-top-card__sticky-header .t-14.truncate")
                .await
            {
                if let Ok(Some(subtitle)) = elem.inner_text().await {
                    if let Some(m) = Self::parse_modality(&subtitle) {
                        modalities.push(m);
                    }
                }
            }
        }

        modalities
    }

    async fn scrape_salary(page: &chromiumoxide::Page) -> Option<Salary> {
        let buttons = page
            .find_elements(".job-details-fit-level-preferences button")
            .await
            .ok()?;

        for button in &buttons {
            let text = button.inner_text().await.ok()??.trim().to_string();

            let currency = if text.contains('€') {
                "EUR"
            } else if text.contains('£') {
                "GBP"
            } else if text.contains('$') {
                "USD"
            } else if text.contains("MUR") {
                "MUR"
            } else {
                continue;
            };

            // Split on " - " to check if it's a range
            let parts: Vec<&str> = text.splitn(2, " - ").collect();

            let minimum = Self::parse_money(parts[0], currency)?;

            let maximum = if parts.len() == 2 {
                Self::parse_money(parts[1], currency) // None if parsing fails
            } else {
                None // single value, no range
            };

            return Some(Salary { minimum, maximum });
        }

        None
    }

    fn parse_money(s: &str, currency: &str) -> Option<Money> {
        let digits: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        let mut value = digits.parse::<f64>().ok()?;

        if s.to_uppercase().contains('K') {
            value *= 1000.0;
        }

        Some(Money {
            value,
            currency: currency.to_string(),
        })
    }

    fn parse_seniority(s: &str) -> ExperienceLevel {
        match s.to_lowercase().as_str() {
            s if s.contains("internship") => ExperienceLevel::Internship,
            s if s.contains("entry") => ExperienceLevel::EntryLevel,
            s if s.contains("associate") => ExperienceLevel::Associate,
            s if s.contains("mid") => ExperienceLevel::MidLevel,
            s if s.contains("senior") => ExperienceLevel::SeniorLevel,
            s if s.contains("director") => ExperienceLevel::Director,
            s if s.contains("executive") => ExperienceLevel::Executive,
            _ => ExperienceLevel::NotApplicable,
        }
    }

    async fn salary_in_range(salary: &Salary, minimum_salary: &Money) -> bool {
        // @todo factor in currency conversion
        salary.minimum.value >= minimum_salary.value
    }

    async fn scrape_job(card: &Element, page: &Page) -> anyhow::Result<JobDetails> {
        let href = card
            .find_element("a.job-card-container__link")
            .await?
            .attribute("href")
            .await?
            .unwrap_or_default();

        // "/jobs/view/4376808572/?..." → "4376808572"
        let id = href
            .trim_start_matches('/')
            .split('/')
            .nth(2) // jobs / view / {id}
            .and_then(|seg| seg.split('?').next()) // strip query string
            .unwrap_or_default()
            .to_string();

        let url = format!("https://www.linkedin.com/jobs/view/{}/", id);

        let title = Self::get_text(page, ".job-details-jobs-unified-top-card__job-title")
            .await
            .unwrap_or_default();

        let company = Self::get_text(page, ".job-details-jobs-unified-top-card__company-name")
            .await
            .unwrap_or_default();

        let location = Self::get_text(page, ".t-black--light.mt2.job-details-jobs-unified-top-card__tertiary-description-container > span > :first-child")
            .await
            .unwrap_or_default();

        let posted_at = Self::get_text(page, ".t-black--light.mt2.job-details-jobs-unified-top-card__tertiary-description-container > span > :nth-child(3)")
            .await
            .unwrap_or_default();

        let description = Self::get_text(page, ".jobs-description__content")
            .await
            .unwrap_or_default();

        let recruiter_name =
            Self::get_text(page, ".hirer-card__hirer-information > a strong").await;

        let modalities = Self::scrape_modalities(page).await;
        let mut seniority = ExperienceLevel::NotApplicable;
        let mut industry: Option<String> = None;

        let insight_els = page
            .find_elements(".job-details-jobs-unified-top-card__job-insight span")
            .await
            .unwrap_or_default();

        for el in &insight_els {
            if let Ok(Some(text)) = el.inner_text().await {
                let t = text.trim().to_string();
                // Seniority keywords
                if t.to_lowercase().contains("level")
                    || t.to_lowercase().contains("senior")
                    || t.to_lowercase().contains("intern")
                    || t.to_lowercase().contains("director")
                    || t.to_lowercase().contains("executive")
                    || t.to_lowercase().contains("associate")
                {
                    seniority = Self::parse_seniority(&t);
                    continue;
                }

                // Anything left that looks like an industry
                if industry.is_none() && !t.is_empty() {
                    industry = Some(t);
                }
            }
        }

        let salary = Self::scrape_salary(page).await;

        Ok(JobDetails {
            id,
            url,
            title,
            description,
            company,
            location,
            posted_at,
            modalities,
            seniority,
            industry,
            salary,
            recruiter_name,
        })
    }
}

#[async_trait]
impl JobDirectory for LinkedinJobDirectory {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "LinkedIn"
    }

    async fn authenticate(&self, _browser: &mut Browser) -> Result<()> {
        Ok(())
    }

    async fn fetch_jobs(
        &self,
        request: FetchJobRequest,
        browser: &mut Browser,
    ) -> Result<Vec<JobEntry>> {
        let page = self.open_jobs_page(&request, browser).await?;
        let mut jobs: Vec<JobEntry> = Vec::new();

        // @todo make this into an iterator

        'scraper: loop {
            let elements = page
                .find_elements("li[data-occludable-job-id]")
                .await
                .map_err(io::Error::other)?;

            for element in elements {
                element.click().await.map_err(io::Error::other)?;
                tokio::time::sleep(Duration::from_secs(1)).await;

                let job = Self::scrape_job(&element, &page)
                    .await
                    .map_err(io::Error::other)?;

                let is_allowable_salary = {
                    if let Some(min_salary) = &request.filters.minimum_salary {
                        if let Some(salary) = &job.salary {
                            Self::salary_in_range(salary, min_salary).await
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                };

                // @todo change this to a fuzzy match
                let org_not_blacklisted = if let Some(blacklist) = &request.filters.org_blacklist {
                    !blacklist.contains(&job.company)
                } else {
                    true
                };

                if is_allowable_salary && org_not_blacklisted {
                    jobs.push(job.into());
                }

                if jobs.len() >= request.limit {
                    break 'scraper;
                }
            }

            // go to next page
            if let Ok(next_button) = page
                .find_element("button[aria-label='View next page']")
                .await
            {
                if let None = next_button.attribute("disabled").await.unwrap_or(None) {
                    println!("Going to next page...");
                    next_button.click().await.map_err(io::Error::other)?;

                    // Wait for new job cards to load
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            }

            // either no next page or we've reached the limit
            break 'scraper;
        }

        Ok(jobs)
    }

    async fn apply_to_job(
        &self,
        request: ApplyRequest,
        browser: &mut Browser,
    ) -> Result<ApplyResult> {
        // open application page
        let application_page = format!("https://www.linkedin.com/jobs/view/{}", request.job_id);
        let page = browser
            .new_page(&application_page)
            .await
            .map_err(io::Error::other)?;

        let script = include_str!("scrappy.js");
        page.evaluate(script).await.map_err(|e| {
            println!("Error: {e}");
            io::Error::other(e)
        })?;

        page.evaluate(format!(
            "window.scrappy.executeCommand('OPEN_APPLICATION_MODAL', {{ jobId: '{}' }})",
            request.job_id
        ))
        .await
        .map_err(|e| {
            println!("Error: {e}");
            io::Error::other(e)
        })?;

        let mut missing_required_infos = Vec::new();

        'page_loop: loop {
            page.evaluate(format!("window.scrappy.executeCommand('INIT_ITERATOR')"))
                .await
                .map_err(|e| {
                    println!("Error: {e}");
                    io::Error::other(e)
                })?;

            'question_loop: loop {
                let result = page
                    .evaluate(format!("window.scrappy.executeCommand('NEXT_QUESTION')",))
                    .await
                    .map_err(|e| {
                        println!("Error: {e}");
                        io::Error::other(e)
                    })?;

                println!("got here");

                let question = match result.value() {
                    Some(q) => match serde_json::from_value::<Question>(q.clone()) {
                        Ok(q) => q,
                        Err(e) => {
                            println!("Error parsing question: {e} {q:?}");
                            panic!("failed")
                        }
                    },
                    None => break 'question_loop,
                };

                let answer = request
                    .question_handler
                    .answer(&question)
                    .await
                    .map_err(io::Error::other)?;

                if let Answer::MissingRequiredInfo = &answer {
                    missing_required_infos.push(question.clone());
                    continue;
                }

                page
                    .evaluate(format!(
                        "window.scrappy.executeCommand('ANSWER_QUESTION', {{ handle: '{}', answer: {} }})",
                        question.id,
                        serde_json::json!(answer)
                    ))
                    .await
                    .map_err(|e| {
                        println!("Error: {e}");
                        io::Error::other(e)
                    })?;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;

            let result = page
                .evaluate(format!("window.scrappy.executeCommand('NEXT_PAGE')",))
                .await
                .map_err(|e| {
                    println!("Error: {e}");
                    io::Error::other(e)
                })?;

            println!("checking if next page {:?}", result.clone().value());

            tokio::time::sleep(Duration::from_secs(1)).await;

            let has_next_page = match result.value() {
                Some(v) => serde_json::from_value::<bool>(v.clone()).unwrap(),
                None => false,
            };

            if !has_next_page {
                break 'page_loop;
            }
        }

        println!("missing_infos: {:?}", missing_required_infos);

        // @todo screenshot review application page and click on apply button

        Ok(ApplyResult::Applied {
            job_id: "".to_string(),
            applied_at: Utc::now(),
        })
    }
}
