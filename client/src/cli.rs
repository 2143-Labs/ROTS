use bevy::ecs::system::Resource;
use clap::ValueEnum;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug, Clone, ValueEnum)]
pub enum Optimizations {
    /// Disable the worldgen and spawn a static floor at 0.0
    NoFloor,
}

#[derive(clap::Parser, Resource, Debug)]
pub struct CliArgs {
    /// Automatically connect to this ip and port (no name resolution, must be an ip.)
    #[arg(short, long, name = "IP")]
    pub autoconnect: Option<String>,

    /// Override your config name when connecting (useful for multiboxing)
    #[arg(short, long, name = "NAME")]
    pub name_override: Option<String>,

    /// Disable some expensive features for debug builds
    #[arg(short, long)]
    pub opts: Vec<Optimizations>,

    /// Print binds and exit
    #[arg(long)]
    pub print_binds: bool,

    /// Print default config and exit
    #[arg(long)]
    pub print_config: bool,
}

impl CliArgs {
    pub fn optimize_floor(&self) -> bool {
        self.opts.contains(&Optimizations::NoFloor)
    }
}
