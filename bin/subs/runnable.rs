use anyhow::Result;
/// This allows for a combined definition of arguments required and entry logic for a given subcommand

pub trait RunnableSubcommand {
    async fn run(&self) -> Result<()>;
}
