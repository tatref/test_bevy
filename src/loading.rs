use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [AssetLoader] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at https://bevy-cheatbook.github.io/features/assets.html
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::Loading)
            .with_collection::<FontAssets>()
            .with_collection::<AudioAssets>()
            .with_collection::<TextureAssets>()
            .continue_to_state(GameState::Menu)
            .build(app);
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see https://github.com/NiklasEi/bevy_asset_loader)

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection)]
pub struct TextureAssets {
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 20,
        rows = 2,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "textures/IconPack 1.1/AllItems/MiscellaneousSource/MiscellaneousOutline.png")]
    pub misc: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 24,
        rows = 16,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(
        path = "textures/heroic-creature-pack 2/HAS Creature Pack 1.2/Necromancer/NecromancerSpriteSheet.png"
    )]
    pub necromancer: Handle<TextureAtlas>,
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 20,
        rows = 16,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(
        path = "textures/heroic-creature-pack 2/HAS Creature Pack 1.2/Castle/CastleSpriteSheet.png"
    )]
    pub castle: Handle<TextureAtlas>,
    #[asset(texture_atlas(
        tile_size_x = 24.,
        tile_size_y = 24.,
        columns = 24,
        rows = 10,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "textures/has-magic-book/HAS Magic Book 1.1/MagicSpellsAllSprites.png")]
    pub magic: Handle<TextureAtlas>,
}
