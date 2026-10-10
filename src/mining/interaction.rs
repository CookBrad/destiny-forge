use bevy::prelude::*;

use crate::core::{GameState, PlayerProfile, ProfileDirty, ToolEnergy};
use crate::graphics::INTERACT_DISTANCE;
use crate::items::Inventory;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer};
use crate::overworld::setup::OverworldEntry;
use crate::ui::interaction_prompt::{best_prompt, InteractionPrompt, PromptKind};
use crate::ui::inventory_window::InventoryWindowOpen;

use super::layout::{ore_tint, MineLadder, OreNode};
use super::ore::{
    mark_node_depleted, reason_swing_cannot_land, swing_pickaxe, OreKind, PickaxeTier, SwingOutcome,
};
use super::swing::{spawn_pickaxe_swing, PickaxeSwing, PickaxeSwingLanded};

const ORE_NODE_REACH: f32 = INTERACT_DISTANCE * 1.2;
const LADDER_REACH: f32 = INTERACT_DISTANCE * 1.5;
const MAX_CRACK_DARKNESS: f32 = 0.6;
const CRACKED_ROCK_COLOR: Color = Color::srgb(0.2, 0.18, 0.2);

pub fn mine_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<&Transform, With<OverworldPlayer>>,
    ladders: Query<&Transform, With<MineLadder>>,
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
        && near_ladder(position, &ladders)
    {
        commands.insert_resource(OverworldEntry::MineReturn);
        next_state.set(GameState::Overworld);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }
}

pub fn start_pickaxe_swing_at_nearest_ore(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    energy: Res<ToolEnergy>,
    inventory: Res<Inventory>,
    player: Query<&Transform, With<OverworldPlayer>>,
    nodes: Query<(Entity, &Transform, &OreNode)>,
    swings_in_progress: Query<(), With<PickaxeSwing>>,
) {
    if !(keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left)) {
        return;
    }
    if !swings_in_progress.is_empty() {
        return;
    }
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let position = player_transform.translation.truncate();

    let Some((node_entity, node_transform, node)) = nodes
        .iter()
        .filter(|(_, node_transform, _)| {
            position.distance(node_transform.translation.truncate()) <= ORE_NODE_REACH
        })
        .min_by(|first, second| {
            let first_distance = position.distance(first.1.translation.truncate());
            let second_distance = position.distance(second.1.translation.truncate());
            first_distance.total_cmp(&second_distance)
        })
    else {
        return;
    };

    let pickaxe = PickaxeTier::best_owned(&inventory);
    if let Some(blocked) =
        reason_swing_cannot_land(node.kind, node.hits_taken, pickaxe, &energy, &inventory)
    {
        report_swing_outcome(blocked, node.kind);
        return;
    }

    let node_is_left_of_player = node_transform.translation.x < player_transform.translation.x;
    spawn_pickaxe_swing(
        &mut commands,
        player_transform,
        node_entity,
        node_is_left_of_player,
    );
}

pub fn land_pickaxe_swing_on_ore_node(
    mut commands: Commands,
    mut energy: ResMut<ToolEnergy>,
    mut inventory: ResMut<Inventory>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
    mut landed_swings: EventReader<PickaxeSwingLanded>,
    mut nodes: Query<(&mut OreNode, &mut Sprite)>,
) {
    for landed in landed_swings.read() {
        let Ok((mut node, mut sprite)) = nodes.get_mut(landed.target_node) else {
            continue;
        };
        let pickaxe = PickaxeTier::best_owned(&inventory);
        let kind = node.kind;
        let outcome = swing_pickaxe(
            kind,
            &mut node.hits_taken,
            pickaxe,
            &mut energy,
            &mut inventory,
        );

        match outcome {
            SwingOutcome::Cracked { hits_taken } => {
                let crack_darkness =
                    hits_taken as f32 / kind.hits_to_break() as f32 * MAX_CRACK_DARKNESS;
                sprite.color = ore_tint(kind).mix(&CRACKED_ROCK_COLOR, crack_darkness);
            }
            SwingOutcome::BrokeAndDroppedOre => {
                mark_node_depleted(&mut profile, node.id);
                dirty.mark();
                commands.entity(landed.target_node).try_despawn_recursive();
            }
            _ => {}
        }
        report_swing_outcome(outcome, kind);
    }
}

fn report_swing_outcome(outcome: SwingOutcome, kind: OreKind) {
    match outcome {
        SwingOutcome::Cracked { .. } => {}
        SwingOutcome::BrokeAndDroppedOre => info!("Mined 1 {} ore.", kind.label()),
        SwingOutcome::TooHard => info!("This {} vein needs an Iron Pickaxe.", kind.label()),
        SwingOutcome::NotEnoughEnergy => info!("Too tired to mine {} — sleep first.", kind.label()),
        SwingOutcome::NoPickaxe => info!("You need a pickaxe."),
        SwingOutcome::NoRoomForOre => info!("Inventory full."),
    }
}

pub fn update_mine_interaction_prompt(
    inventory_open: Res<InventoryWindowOpen>,
    inventory: Res<Inventory>,
    player: Query<&Transform, With<OverworldPlayer>>,
    ladders: Query<&Transform, With<MineLadder>>,
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
    if near_ladder(position, &ladders) {
        candidates.push(PromptKind::LeaveMine);
    }
    let nearest_ore_in_reach = nodes
        .iter()
        .map(|(node_transform, node)| {
            (
                position.distance(node_transform.translation.truncate()),
                node.kind,
            )
        })
        .filter(|(distance, _)| *distance <= ORE_NODE_REACH)
        .min_by(|first, second| first.0.total_cmp(&second.0));
    if let Some((_, kind)) = nearest_ore_in_reach {
        let pickaxe = PickaxeTier::best_owned(&inventory);
        candidates.push(match kind {
            OreKind::Copper => PromptKind::MineCopper,
            OreKind::Iron if pickaxe.is_some_and(|owned| owned.can_mine(OreKind::Iron)) => {
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

fn near_ladder(position: Vec2, ladders: &Query<&Transform, With<MineLadder>>) -> bool {
    ladders
        .iter()
        .any(|ladder| position.distance(ladder.translation.truncate()) <= LADDER_REACH)
}
