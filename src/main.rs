use earthquake::{
    builder::CheckerBuilder,
    checker::{CheckModule, CheckerState},
    combo::Combo,
    proxy::Proxy,
    result::CheckResult,
    stats::Stats,
};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

struct SimpleModule;

#[async_trait::async_trait]
impl CheckModule for SimpleModule {
    fn name(&self) -> &str {
        "simple"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn author(&self) -> &str {
        "PenTech"
    }

    fn description(&self) -> &str {
        "A simple example module for demonstration"
    }

    async fn check(
        &self,
        _client: Arc<Client>,
        combo: Combo,
        _proxy: Option<Proxy>,
    ) -> CheckResult {
        sleep(Duration::from_millis(100)).await;

        let hash = combo.username.chars().map(|c| c as u32).sum::<u32>();

        let login_ip = format!("192.168.1.{}", hash % 255);
        let points = (hash % 1000).to_string();
        let last_login = format!("2023-{:02}-{:02}", (hash % 12) + 1, (hash % 28) + 1);
        let subscription = if hash % 4 == 0 { "Premium" } else { "Basic" };

        if hash % 10 == 0 {
            CheckResult::hit()
                .with_capture("lastLoginIp", login_ip)
                .with_capture("points", points)
                .with_capture("lastLogin", last_login)
                .with_capture("subscription", subscription)
        } else if hash % 10 == 1 {
            CheckResult::free()
                .with_capture("lastLoginIp", login_ip)
                .with_capture("points", "0")
        } else if hash % 10 == 2 {
            CheckResult::invalid()
        } else if hash % 10 == 3 {
            CheckResult::banned().with_capture("banReason", "Too many attempts")
        } else if hash % 10 == 4 {
            CheckResult::retry()
        } else {
            CheckResult::error()
        }
    }
}

async fn display_stats(checker: Arc<earthquake::checker::Checker>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let stats = checker.get_stats().await;
        let state = checker.get_state().await;

        if state == CheckerState::Finished {
            println!("Checker finished!");
            break;
        }

        println!("\n--- Stats ---");
        println!("State: {:?}", state);
        println!("Progress: {:.2}%", stats.progress());
        println!("Checked: {}/{}", stats.checked(), stats.total());
        println!("Hits: {}", stats.hits());
        println!("Free: {}", stats.free());
        println!("Errors: {}", stats.errors());
        println!("Invalid: {}", stats.invalid());
        println!("Banned: {}", stats.banned());
        println!("Retries: {}", stats.retries());
        println!("CPM: {}", stats.cpm());
        println!("Elapsed: {}", Stats::format_duration(stats.elapsed()));
        println!("ETA: {}", Stats::format_duration(stats.eta()));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    create_sample_files()?;

    let module = Arc::new(SimpleModule);

    let builder = CheckerBuilder::new("simple_demo")
        .with_threads(10)
        .with_max_retries(3)
        .with_combo_file("data/combos.txt")?
        .with_check_module(module);

    let checker = Arc::new(builder.build()?);

    let stats_handle = {
        let checker_clone = checker.clone();
        tokio::spawn(async move {
            display_stats(checker_clone).await;
        })
    };

    checker.start().await?;

    stats_handle.await?;

    println!("All done!");
    println!("Results saved to results/simple_demo/ directory");

    Ok(())
}

fn create_sample_files() -> std::io::Result<()> {
    use std::fs::{self, File};
    use std::io::Write;

    fs::create_dir_all("data")?;

    let mut file = File::create("data/combos.txt")?;

    for i in 1..=100 {
        writeln!(file, "user{}:password{}", i, i)?;
    }

    Ok(())
}
