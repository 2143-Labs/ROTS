use bevy::ecs::system::Resource;

#[derive(clap::Parser, Resource)]
pub struct CliArgs {
    #[arg(short, long)]
    pub autoconnect: Option<String>,
    #[arg(short, long)]
    pub name_override: Option<String>,
}
