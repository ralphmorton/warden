use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Address to listen on
    #[arg(long = "addr", default_value = "::1")]
    pub addr: String,

    /// Port to listen on
    #[arg(long = "port")]
    pub port: u16,

    /// Scheme for default downstream server
    #[arg(long = "ds-scheme")]
    pub downstream_scheme: String,

    /// Host for default downstream server
    #[arg(long = "ds-host")]
    pub downstream_host: String,

    /// Port for default downstream server
    #[arg(long = "ds-port")]
    pub downstream_port: Option<u16>,

    /// Log level
    #[arg(long = "log", default_value = "debug")]
    pub log_level: String,
}
