use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(default_value = "./config.toml", short, long)]
    pub config: PathBuf,
}