
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    name: String,
}


fn main() {
    let args = Args::parse();
    println!("Deploying Headstash Framework As Governance Module, {}!", args.name);
}
