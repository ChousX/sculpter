// Example: How to use the Surface Nets plugin
use bevy::prelude::*;
use sculpter::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Use your app plugin configuration if different
        .add_plugin(SculpterPlugin)
        .run();
}
