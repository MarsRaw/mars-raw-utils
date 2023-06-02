use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use clap::Parser;
use mars_raw_utils::caldata;
pb_create!();

#[derive(Parser)]
#[command(author, version, about = "Updated calibration data from remote repository", long_about = None)]
pub struct UpdateCalData {
    #[arg(long, short, help = "Do not replace existing files")]
    noreplace: bool,

    #[arg(long, short, help = "Override default storage path")]
    local_store: Option<String>,
}

#[async_trait::async_trait]
impl RunnableSubcommand for UpdateCalData {
    async fn run(&self) -> Result<()> {
        pb_set_print!();
        match caldata::update_calibration_data(
            !self.noreplace,
            &self.local_store,
            |t| {
                pb_set_length!(t);
            },
            || pb_inc!(),
        )
        .await
        {
            Ok(_) => println!("Done."),
            Err(why) => println!("Error: {}", why),
        };

        Ok(())
    }
}
