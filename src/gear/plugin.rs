use bevy::prelude::*;

use super::GearCatalog;

pub struct GearPlugin;

impl Plugin for GearPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GearCatalog>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gear_plugin_registers_the_catalog_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(GearPlugin);
        app.update();
        let catalog = app.world().resource::<GearCatalog>();
        assert_eq!(catalog.attack("rusty_sword"), 10.0);
        assert_eq!(catalog.name("slime_blade"), "Slime Blade");
    }
}
