pub use super::super::Item;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Corn;

impl Item for Corn {
    fn name(&self) -> &'static str {
        "Corn"
    }

    fn inventory_image(&self) -> &'static str {
        "crop.png"
    }

    fn stack_size(&self) -> usize {
        64
    }

    fn id(&self) -> u32 {
        1
    }
}
