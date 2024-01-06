use crate::stats::Health;
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ShootingData {
    pub shot_from: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateSharedComponent {
    Health(Health),
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum, Component, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum NPC {
    Penguin,
    Mage
}

#[derive(Component)]
pub enum AIType {
    None,
    WalkToNearestPlayer,
}

// Client side only
#[derive(Resource)]
pub struct NPCAnimations(HashMap<&'static str, Handle<AnimationClip>>);

impl NPCAnimations {
    pub fn get_anim(&self, name: &str) -> Handle<AnimationClip> {
        self.0.get(name).unwrap().clone_weak()
    }
}

impl NPC {
    pub fn model(&self) -> &'static str {
        match self {
            NPC::Penguin => "penguinwalk.gltf#Scene0",
            NPC::Mage => "bookmageIdle.gltf#Scene0",
        }
    }

    pub fn get_base_health(&self) -> Health {
        Health(match self {
            NPC::Penguin => 50,
            NPC::Mage => 20
        })
    }

    pub fn get_ai_component(&self) -> AIType {
        match self {
            NPC::Penguin => AIType::WalkToNearestPlayer,
            NPC::Mage => AIType::WalkToNearestPlayer
        }
    }

    pub fn animation(&self) -> &'static str {
        match self {
            NPC::Penguin => "penguinwalk.gltf#Animation0",
            NPC::Mage => "bookmageIdle.gltf#Animation0",
        }
    }

    pub fn load_all_animations(
        commands: &mut Commands,
        asset_server: &AssetServer,
    ) {
        let mut hashes = HashMap::new();
        for npc in [NPC::Penguin, NPC::Mage] {
            let anim_name = npc.animation();
            hashes.insert(anim_name, asset_server.load(npc.animation()));
        }
        commands.insert_resource(NPCAnimations(hashes));

    }
}
