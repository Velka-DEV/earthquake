use crate::checker::CheckResultCallback;
use crate::checker::{CheckFunction, CheckModule, Checker};
use crate::combo::{ComboProvider, FileComboProvider};
use crate::config::Config;
use crate::proxy::{FileProxyProvider, ProxyProvider};
use crate::result::CheckResult;
use crate::{Combo, Result};
use futures::Future;
use reqwest::Client;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub struct CheckerBuilder {
    config: Config,
    combo_provider: Option<Arc<dyn ComboProvider>>,
    proxy_provider: Option<Arc<dyn ProxyProvider>>,
    check_fn: Option<CheckFunction>,
    check_result_callback: Option<CheckResultCallback>,
}

impl CheckerBuilder {
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            config: Config::new(module_name),
            combo_provider: None,
            proxy_provider: None,
            check_fn: None,
            check_result_callback: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.config.threads = threads;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn with_proxy_cooldown(mut self, cooldown: Duration) -> Self {
        self.config.proxy_cooldown = cooldown;
        self
    }

    pub fn with_save_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.save_dir = dir.into();
        self
    }

    pub fn with_combo_provider(mut self, provider: Arc<dyn ComboProvider>) -> Self {
        self.combo_provider = Some(provider);
        self
    }

    pub fn with_combo_file(self, path: impl Into<String>) -> Result<Self> {
        let separator = self.config.combo_separator.clone();
        let mut provider = FileComboProvider::new(path).with_separator(separator);

        if let Some(pattern) = &self.config.combo_regex_filter {
            provider = provider.with_regex_filter(pattern)?;
        }

        provider.load()?;

        Ok(self.with_combo_provider(Arc::new(provider)))
    }

    pub fn with_proxy_provider(mut self, provider: Arc<dyn ProxyProvider>) -> Self {
        self.proxy_provider = Some(provider);
        self
    }

    pub fn with_proxy_file(self, path: impl Into<String>) -> Result<Self> {
        let provider = FileProxyProvider::new()
            .with_cooldown(self.config.proxy_cooldown)
            .with_max_failures(self.config.proxy_max_failures)
            .random(self.config.random_proxies);

        provider.load_from_file(path.into())?;

        Ok(self.with_proxy_provider(Arc::new(provider)))
    }

    pub async fn with_proxy_url(self, url: impl Into<String>) -> Result<Self> {
        let provider = FileProxyProvider::new()
            .with_cooldown(self.config.proxy_cooldown)
            .with_max_failures(self.config.proxy_max_failures)
            .random(self.config.random_proxies);

        provider.load_from_url(&url.into()).await?;

        Ok(self.with_proxy_provider(Arc::new(provider)))
    }

    pub fn with_check_function<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Arc<Client>, crate::combo::Combo, Option<crate::proxy::Proxy>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = CheckResult> + Send + 'static,
    {
        let check_fn = Arc::new(move |client, combo, proxy| {
            let future = f(client, combo, proxy);
            Box::pin(future) as Pin<Box<dyn Future<Output = CheckResult> + Send>>
        });

        self.check_fn = Some(check_fn);
        self
    }

    pub fn with_check_module(self, module: Arc<dyn CheckModule>) -> Self {
        self.with_check_function(move |client, combo, proxy| {
            let module = module.clone();
            async move { module.check(client, combo, proxy).await }
        })
    }

    pub fn with_check_result_callback<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(CheckResult, Combo, Option<crate::proxy::Proxy>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let callback = Arc::new(move |result, combo, proxy| {
            let future = f(result, combo, proxy);
            Box::pin(future) as Pin<Box<dyn Future<Output = ()> + Send>>
        });
        self.check_result_callback = Some(callback);
        self
    }

    pub fn build(self) -> Result<Checker> {
        let mut checker = Checker::new(self.config);

        if let Some(provider) = self.combo_provider {
            checker.with_combo_provider(provider);
        }

        if let Some(provider) = self.proxy_provider {
            checker.with_proxy_provider(provider);
        }

        if let Some(check_fn) = self.check_fn {
            checker.with_check_function(check_fn);
        }

        if let Some(callback) = self.check_result_callback {
            checker.with_check_result_callback(callback);
        }

        Ok(checker)
    }
}
