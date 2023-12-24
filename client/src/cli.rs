use bevy::ecs::system::Resource;
use clap::ValueEnum;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug, Clone, ValueEnum)]
pub enum Optimizations {
    NoFloor,
}

#[derive(clap::Parser, Resource, Debug)]
pub struct CliArgs {
    #[arg(short, long)]
    pub autoconnect: Option<String>,
    #[arg(short, long)]
    pub name_override: Option<String>,

    #[arg(short, long)]
    pub opts: Vec<Optimizations>,
}

impl CliArgs {
    pub fn optimize_floor(&self) -> bool {
        self.opts.contains(&Optimizations::NoFloor)
    }
}
