/// This allows for a combined definition of arguments required and entry logic for a given subcommand
pub trait RunnableSubcommand {
    fn run(&self);
}
