use clap::{Parser, arg, command};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Run without the UI
   #[arg(long)]
   pub headless: bool
}