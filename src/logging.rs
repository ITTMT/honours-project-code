use std::fmt::Display;

use tower_lsp::lsp_types::MessageType;

use crate::Backend;

pub trait Logging {
    async fn log_info<M: Display>(&self, message: M);

    async fn log_error<M: Display>(&self, message: M);
}

impl Logging for Backend {
    async fn log_info<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::INFO, message).await;
    }

    async fn log_error<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::ERROR, message).await;
    }
}
