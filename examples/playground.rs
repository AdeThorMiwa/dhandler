use di::ServiceProvider;
use loco_rs::{app::Hooks, boot::create_context, environment::Environment};
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use the_handler::{
    app::App,
    models::linkedin_creds::LinkedinCreds,
    services::directory::{
        linkedin::LinkedInAuthCredentials, FetchJobsRequest, JobDirectory, JobSearchFilters,
    },
    utils::{app::create_di_provider, settings::Settings},
};

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

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // let _ctx = playground::<App>().await?;

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

    Ok(())
}
