pub mod error;
pub mod browser;

pub use error::{BrowserError, Result};
pub use browser::{BrowserSession, LaunchOptions, ConnectionOptions};
