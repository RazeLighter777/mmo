use clap::Parser;
#[derive(Parser)]
#[clap(author = "Justin Suess", version, about = "rust ecs mmo server")]
pub struct Args {
    #[clap(short, long, default_value_t = 4200, help = "port to bind service to")]
    pub port: u16,
    #[clap(long, default_value = "127.0.0.1", help = "ip to bind service to")]
    pub ip: String,
    #[clap(
        long,
        default_value = "localhost",
        help = "host to connect to for database (mysql)"
    )]
    pub database_host: String,
    #[clap(long, default_value = "mmo", help = "user to login to database with")]
    pub database_user: String,
    #[clap(
        long,
        default_value = "mmopass",
        help = "password to login to database server with"
    )]
    pub database_pass: String,
    #[clap(
        long,
        default_value = "mmodat",
        help = "name of server database to use"
    )]
    pub database_name: String,
    #[clap(
        long,
        short,
        default_value = "secret",
        help = "name of server database to use"
    )]
    pub secret: String,
}
