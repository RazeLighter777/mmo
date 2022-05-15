use clap::Parser;
#[derive(Parser)]
#[clap(author = "Justin Suess", version, about = "rust ecs mmo server")]
pub struct Args {
    #[clap(short, long, default_value_t = 4200, help = "port to bind service to")]
    pub port: u16,
    #[clap(long, default_value = "127.0.0.1", help = "ip to bind service to")]
    pub ip: String,
}
