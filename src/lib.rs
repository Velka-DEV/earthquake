pub mod builder;
pub mod checker;
pub mod combo;
pub mod config;
pub mod error;
pub mod proxy;
pub mod result;
pub mod stats;
pub mod util;

pub use builder::CheckerBuilder;
pub use checker::Checker;
pub use combo::{Combo, ComboProvider};
pub use config::Config;
pub use error::Error;
pub use proxy::{Proxy, ProxyProvider};
pub use result::{CheckResult, ResultType};

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
