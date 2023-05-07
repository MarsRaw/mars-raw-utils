use mars_raw_utils::caldata;

use crate::subs::runnable::RunnableSubcommand;

#[derive(clap::Args)]
#[clap(author, version, about = "Updated calibration data from remote repository", long_about = None)]
pub struct UpdateCalData {
    #[clap(long, short, help = "Do not replace existing files")]
    noreplace: bool,
}

#[async_trait::async_trait]
impl RunnableSubcommand for UpdateCalData {
    async fn run(&self) {
        match caldata::update_calibration_data(!self.noreplace).await {
            Ok(_) => println!("Done."),
            Err(why) => println!("Error: {}", why),
        };
    }
}
