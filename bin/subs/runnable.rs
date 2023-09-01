use anyhow::Result;
/// This allows for a combined definition of arguments required and entry logic for a given subcommand
use async_trait::async_trait;

#[async_trait]
pub trait RunnableSubcommand {
    async fn run(&self) -> Result<()>;
}
