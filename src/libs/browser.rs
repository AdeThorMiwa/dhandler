use chromiumoxide::Browser;
use futures::StreamExt;

pub struct EnvironmentOrchestrator {
    brower: Browser,
}

impl EnvironmentOrchestrator {
    pub async fn start() -> anyhow::Result<Self> {
        let (browser, mut handler) = Browser::connect("http://127.0.0.1:9222")
            .await
            .map_err(anyhow::Error::from)?;

        // Spawn the event handler loop (required)
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                // handle or ignore events
                let _ = event;
            }
        });

        Ok(Self { brower: browser })
    }

    pub fn get_browser(&mut self) -> &mut Browser {
        &mut self.brower
    }
}
