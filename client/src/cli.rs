use bevy::ecs::system::Resource;

#[derive(clap::Parser, Resource)]
pub struct CliArgs {
    #[arg(short, long)]
    pub autoconnect: String,
}
