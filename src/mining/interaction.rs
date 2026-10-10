use bevy::prelude::*;

use crate::core::{GameState, PlayerProfile, ProfileDirty, ToolEnergy};
use crate::graphics::INTERACT_DISTANCE;
use crate::items::Inventory;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer};
use crate::overworld::setup::OverworldEntry;
use crate::ui::interaction_prompt::{best_prompt, InteractionPrompt, PromptKind};
use crate::ui::inventory_window::InventoryWindowOpen;

use super::layout::{ore_tint, MineExit, OreNode};
use super::ore::{mark_node_depleted, swing_pickaxe, OreKind, PickaxeTier, SwingOutcome};

const NODE_RANGE: f32 = INTERACT_DISTANCE * 1.2;
const EXIT_RANGE: f32 = INTERACT_DISTANCE * 1.5;

pub fn mine_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<&Transform, With<OverworldPlayer>>,
    exits: Query<&Transform, With<MineExit>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let Ok(transform) = player.get_single() else {
        return;
    };
    if inventory.0 {
        return;
    }
    let position = transform.translation.truncate();

    if cooldown.0.finished()
        && keyboard.just_pressed(KeyCode::KeyE)
        && near_exit(position, &exits)
    {
        commands.insert_resource(OverworldEntry::MineReturn);
        next_state.set(GameState::Overworld);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }
}

/// Space / left click swings the best owned pickaxe at the nearest node.
pub fn swing_at_ore_node(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut energy: ResMut<ToolEnergy>,
    mut inventory: ResMut<Inventory>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
    player: Query<&Transform, With<OverworldPlayer>>,
    mut nodes: Query<(Entity, &Transform, &mut OreNode, &mut Sprite)>,
) {
    if !(keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left)) {
        return;
    }
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let position = player_transform.translation.truncate();

    let Some((entity, _, mut node, mut sprite)) = nodes
        .iter_mut()
        .filter(|(_, t, _, _)| position.distance(t.translation.truncate()) <= NODE_RANGE)
        .min_by(|a, b| {
            let da = position.distance(a.1.translation.truncate());
            let db = position.distance(b.1.translation.truncate());
            da.total_cmp(&db)
        })
    else {
        return;
    };

    let pickaxe = PickaxeTier::best_owned(&inventory);
    let kind = node.kind;
    let outcome = swing_pickaxe(kind, &mut node.hits, pickaxe, &mut energy, &mut inventory);

    match outcome {
        SwingOutcome::Hit { hits } => {
            // Darken the vein as it cracks; no new art.
            let worn = hits as f32 / kind.hits_required() as f32 * 0.6;
            sprite.color = ore_tint(kind).mix(&Color::srgb(0.2, 0.18, 0.2), worn);
        }
        SwingOutcome::Broke => {
            mark_node_depleted(&mut profile, node.id);
            dirty.mark();
            commands.entity(entity).try_despawn_recursive();
            info!("Mined 1 {} ore.", kind.label());
        }
        SwingOutcome::TooHard => info!("This {} vein needs an Iron Pickaxe.", kind.label()),
        SwingOutcome::NotEnoughEnergy => info!("Too tired to mine {} — sleep first.", kind.label()),
        SwingOutcome::NoPickaxe => info!("You need a pickaxe."),
        SwingOutcome::InventoryFull => info!("Inventory full."),
    }
}

pub fn update_mine_interaction_prompt(
    inventory_open: Res<InventoryWindowOpen>,
    inventory: Res<Inventory>,
    player: Query<&Transform, With<OverworldPlayer>>,
    exits: Query<&Transform, With<MineExit>>,
    nodes: Query<(&Transform, &OreNode)>,
    mut prompt: ResMut<InteractionPrompt>,
) {
    if inventory_open.0 {
        prompt.clear();
        return;
    }
    let Ok(player_transform) = player.get_single() else {
        prompt.clear();
        return;
    };
    let position = player_transform.translation.truncate();

    let mut candidates = Vec::with_capacity(2);
    if near_exit(position, &exits) {
        candidates.push(PromptKind::LeaveMine);
    }
    let nearest = nodes
        .iter()
        .map(|(t, n)| (position.distance(t.translation.truncate()), n.kind))
        .filter(|(d, _)| *d <= NODE_RANGE)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, kind)) = nearest {
        let pickaxe = PickaxeTier::best_owned(&inventory);
        candidates.push(match kind {
            OreKind::Copper => PromptKind::MineCopper,
            OreKind::Iron if pickaxe.is_some_and(|p| p.can_mine(OreKind::Iron)) => {
                PromptKind::MineIron
            }
            OreKind::Iron => PromptKind::NeedIronPickaxe,
        });
    }
    let kind = best_prompt(&candidates);
    if prompt.kind != kind {
        prompt.set(kind);
    }
}

fn near_exit(position: Vec2, exits: &Query<&Transform, With<MineExit>>) -> bool {
    exits
        .iter()
        .any(|exit| position.distance(exit.translation.truncate()) <= EXIT_RANGE)
}
