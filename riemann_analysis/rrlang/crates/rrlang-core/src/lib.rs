pub mod config;
pub mod corpus;
pub mod encode;
pub mod error;
pub mod metrics;
pub mod nulls;
pub mod pipeline;
pub mod primes;
pub mod report;
pub mod types;

pub use config::load_config_file;
pub use corpus::{inspect_file, CorpusInspection};
pub use error::{Result, RrlangError};
pub use pipeline::run_analysis;
pub use report::{report_to_json, report_to_text};
pub use types::*;
