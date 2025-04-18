use crate::Result;
use crate::combo::{Combo, ComboProvider};
use crate::config::Config;
use crate::error::Error;
use crate::proxy::{Proxy, ProxyProvider};
use crate::result::{CheckResult, ResultStatus};
use crate::stats::Stats;
use crate::util;
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc, watch};

pub type CheckFunction = Arc<
    dyn Fn(Arc<Client>, Combo, Option<Proxy>) -> futures::future::BoxFuture<'static, CheckResult>
        + Send
        + Sync,
>;

pub type ResultCallback = Arc<
    dyn Fn(Combo, CheckResult, Option<Proxy>) -> futures::future::BoxFuture<'static, ()>
        + Send
        + Sync,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckerState {
    Idle,
    Running,
    Paused,
    Stopped,
    Finished,
}

pub struct Checker {
    config: Config,
    check_fn: Option<CheckFunction>,
    combo_provider: Option<Arc<dyn ComboProvider>>,
    proxy_provider: Option<Arc<dyn ProxyProvider>>,
    state: Arc<RwLock<CheckerState>>,
    stats: Arc<RwLock<Stats>>,
    state_notify: Arc<watch::Sender<CheckerState>>,
    state_rx: watch::Receiver<CheckerState>,
    session_start_time: String,
    result_callback: Option<ResultCallback>,
}

impl Checker {
    pub fn new(config: Config) -> Self {
        let (state_tx, state_rx) = watch::channel(CheckerState::Idle);

        Self {
            config,
            check_fn: None,
            combo_provider: None,
            proxy_provider: None,
            state: Arc::new(RwLock::new(CheckerState::Idle)),
            stats: Arc::new(RwLock::new(Stats::new())),
            state_notify: Arc::new(state_tx),
            state_rx,
            session_start_time: util::format_datetime_now(),
            result_callback: None,
        }
    }

    pub fn with_check_function(&mut self, check_fn: CheckFunction) {
        self.check_fn = Some(check_fn);
    }

    pub fn with_combo_provider(&mut self, provider: Arc<dyn ComboProvider>) {
        self.combo_provider = Some(provider);
    }

    pub fn with_proxy_provider(&mut self, provider: Arc<dyn ProxyProvider>) {
        self.proxy_provider = Some(provider);
    }

    pub fn with_result_callback(&mut self, callback: ResultCallback) {
        self.result_callback = Some(callback);
    }

    pub async fn start(&self) -> Result<()> {
        if self.check_fn.is_none() {
            return Err(Error::NoCheckFunction);
        }

        if self.combo_provider.is_none() {
            return Err(Error::NoCombos);
        }

        let mut state = self.state.write().await;
        *state = CheckerState::Running;
        drop(state);
        self.state_notify
            .send(CheckerState::Running)
            .map_err(|_| Error::Thread("Failed to notify state change".to_string()))?;

        let combo_provider = self.combo_provider.as_ref().unwrap();
        let total_combos = combo_provider.len().await;

        let mut stats = self.stats.write().await;
        stats.set_total(total_combos);
        stats.start();
        drop(stats);

        let (result_tx, mut result_rx) = mpsc::channel::<(Combo, CheckResult, Option<Proxy>)>(1000);

        let config_clone = self.config.clone();
        let results_dir = format!(
            "{}/{}/{}",
            config_clone.save_dir, config_clone.module_name, self.session_start_time
        );

        let result_callback = self.result_callback.clone();

        let _result_handler = tokio::spawn(async move {
            if let Err(e) = util::create_directory_if_not_exists(&results_dir) {
                eprintln!("Failed to create results directory: {}", e);
                return;
            }

            let mut result_paths = std::collections::HashMap::new();

            while let Some((combo, result, proxy)) = result_rx.recv().await {
                // Call the callback if provided
                if let Some(ref callback) = result_callback {
                    callback(combo.clone(), result.clone(), proxy.clone()).await;
                }

                let result_type = result.status.to_string();

                let path = result_paths
                    .entry(result.status)
                    .or_insert_with(|| format!("{}/{}.txt", results_dir, result_type));

                let mut content = format!("{}", combo);

                if let Some(ref message) = result.message {
                    content = format!("{} | {}", content, message);
                }

                if !result.captures.is_empty() {
                    let captures_str = result
                        .captures
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join(" - ");
                    content = format!("{} | {}", content, captures_str);
                }

                if let Some(ref data) = result.extra_data {
                    content = format!("{} | {}", content, data);
                }

                if let Err(e) = util::append_to_file(path, &content) {
                    eprintln!("Failed to write result: {}", e);
                }
            }
        });

        let state = self.state.clone();
        let stats = self.stats.clone();
        let check_fn = self.check_fn.clone().unwrap();
        let combo_provider = self.combo_provider.clone().unwrap();
        let proxy_provider = self.proxy_provider.clone();
        let config = self.config.clone();
        let result_tx = Arc::new(result_tx);

        tokio::spawn(async move {
            let max_retries = config.max_retries;

            stream::iter(0..config.threads)
                .for_each_concurrent(config.threads, |_| {
                    let state = state.clone();
                    let stats = stats.clone();
                    let check_fn = check_fn.clone();
                    let combo_provider = combo_provider.clone();
                    let proxy_provider = proxy_provider.clone();
                    let result_tx = result_tx.clone();

                    async move {
                        loop {
                            let current_state = *state.read().await;
                            if current_state == CheckerState::Stopped
                                || current_state == CheckerState::Finished
                            {
                                break;
                            }

                            if current_state == CheckerState::Paused {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            let combo = match combo_provider.next().await {
                                Some(combo) => combo,
                                None => {
                                    break;
                                }
                            };

                            let proxy = if let Some(ref provider) = proxy_provider {
                                provider.next().await
                            } else {
                                None
                            };

                            let client = match util::build_http_client(proxy.as_ref()).await {
                                Ok(client) => Arc::new(client),
                                Err(_) => continue,
                            };

                            let mut result = check_fn(client, combo.clone(), proxy.clone()).await;
                            let mut retry_count = 0;
                            let mut final_proxy = proxy.clone();

                            while result.status == ResultStatus::Retry && retry_count < max_retries
                            {
                                retry_count += 1;

                                if let Some(ref mut proxy) = proxy.clone() {
                                    proxy.mark_failure();
                                }

                                tokio::time::sleep(Duration::from_millis(500)).await;

                                let new_proxy = if let Some(ref provider) = proxy_provider {
                                    provider.next().await
                                } else {
                                    None
                                };

                                final_proxy = new_proxy.clone();

                                match util::build_http_client(new_proxy.as_ref()).await {
                                    Ok(new_client) => {
                                        result = check_fn(
                                            Arc::new(new_client),
                                            combo.clone(),
                                            new_proxy.clone(),
                                        )
                                        .await;
                                    }
                                    Err(_) => continue,
                                }
                            }

                            stats.write().await.increment_checked();
                            stats.write().await.increment_result(result.status);

                            let result = result.with_retry_count(retry_count);
                            if let Err(_) = result_tx.send((combo, result, final_proxy)).await {
                                break;
                            }
                        }
                    }
                })
                .await;

            let mut state = state.write().await;
            *state = CheckerState::Finished;
            drop(state);
        });

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if *state == CheckerState::Running {
            *state = CheckerState::Paused;
            self.state_notify
                .send(CheckerState::Paused)
                .map_err(|_| Error::Thread("Failed to notify state change".to_string()))?;
            self.stats.write().await.pause();
        }

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if *state == CheckerState::Paused {
            *state = CheckerState::Running;
            self.state_notify
                .send(CheckerState::Running)
                .map_err(|_| Error::Thread("Failed to notify state change".to_string()))?;
            self.stats.write().await.start();
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if *state == CheckerState::Running || *state == CheckerState::Paused {
            *state = CheckerState::Stopped;
            self.state_notify
                .send(CheckerState::Stopped)
                .map_err(|_| Error::Thread("Failed to notify state change".to_string()))?;
        }

        Ok(())
    }

    pub async fn save_remaining(&self, _path: impl AsRef<Path>) -> Result<usize> {
        if let Some(_provider) = &self.combo_provider {
            // This is a design limitation; the ComboProvider trait doesn't provide save_remaining method
            // We'd need to implement a way to access concrete types or add this method to the trait

            Err(Error::Unknown(
                "Save remaining not implemented yet".to_string(),
            ))
        } else {
            Err(Error::NoCombos)
        }
    }

    pub async fn get_stats(&self) -> Stats {
        self.stats.read().await.clone()
    }

    pub async fn get_state(&self) -> CheckerState {
        *self.state.read().await
    }

    pub fn watch_state(&self) -> watch::Receiver<CheckerState> {
        self.state_rx.clone()
    }
}

#[async_trait]
pub trait CheckModule: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    async fn check(&self, client: Arc<Client>, combo: Combo, proxy: Option<Proxy>) -> CheckResult;
}
