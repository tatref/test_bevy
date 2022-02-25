use std::time::Duration;

use crate::loading::TextureAssets;
use crate::player::{BaseMoveSpeed, Health, HitBox, HurtBox, Player};
use crate::{GameConfiguration, GameState};
use benimator::{Play, SpriteSheetAnimation};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

pub struct EnemyPlugin;

#[derive(Component)]
pub struct Corpse {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Alive;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemySpawner {
    timer: Timer,
    times: u32,
    bag: u32,
    spread: f32,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(enemy_setup))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(spawner_tick))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(move_enemy));
    }
}

fn spawner_tick(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    assets: Res<Assets<TextureAtlas>>,
    mut spawner: Query<&mut EnemySpawner>,
    time: Res<Time>,
    config: Res<GameConfiguration>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    let mut rng = rand::thread_rng();
    use rand::prelude::*;

    let animation_handle = animations.add(
        SpriteSheetAnimation::from_range(
            28..=31,                            // Indices of the sprite atlas
            Duration::from_secs_f64(1.0 / 4.0), // Duration of each frame
        )
        .repeat(),
    );

    for mut spawner in spawner.iter_mut() {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() && spawner.times > 0 {
            spawner.times -= 1;

            let t = rng.gen_range(0f32..(std::f32::consts::PI * 2.));
            let r = 300.;
            for _ in 0..spawner.bag {
                let dx = rng.gen_range((-spawner.spread / 2.)..(spawner.spread / 2.));
                let dy = rng.gen_range((-spawner.spread / 2.)..(spawner.spread / 2.));
                let x = r * t.sin() + dx;
                let y = r * t.cos() + dy;

                let texture = assets.get(&textures.necromancer).unwrap();
                let size = texture.textures[0].size();

                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: textures.necromancer.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            x,
                            y,
                            sprite_z(Vec2::new(x, y)),
                        ))
                        .with_scale(Vec3::splat(config.scale)),
                        sprite: TextureAtlasSprite::new(27),

                        ..Default::default()
                    })
                    .insert(animation_handle.clone())
                    .insert(Play)
                    .insert(Enemy)
                    .insert(Alive)
                    .insert(BaseMoveSpeed(10.))
                    .insert(HurtBox {
                        pos: Vec2::ZERO,
                        size,
                    })
                    .insert(Health(3.))
                    .insert(HitBox {
                        pos: Vec2::ZERO,
                        size,
                        damage: 1.,
                    });
            }
        }
    }
}

fn enemy_setup(mut commands: Commands) {
    commands.spawn().insert(EnemySpawner {
        timer: Timer::new(Duration::from_secs_f32(2.0), true),
        times: 100,
        bag: 5,
        spread: 50.,
    });
}

fn move_enemy(
    time: Res<Time>,
    mut enemy_query: Query<
        (&mut Transform, &BaseMoveSpeed, &mut TextureAtlasSprite),
        (With<Enemy>, With<Alive>, Without<Player>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    let player = player_query.single();

    for (mut enemy_transform, base_speed, mut enemy_sprite) in enemy_query.iter_mut() {
        let delta = (player.translation - enemy_transform.translation).clamp_length_max(1.);
        let movement = delta * base_speed.0 * time.delta_seconds();

        if movement.x <= 0. {
            enemy_sprite.flip_x = true;
        } else {
            enemy_sprite.flip_x = false;
        }

        enemy_transform.translation += movement;

        // Z draw order
        enemy_transform.translation.z = sprite_z(enemy_transform.translation.xy());
    }
}

pub fn sprite_z(xy: Vec2) -> f32 {
    let base_z = 1.;

    base_z - xy.y / 10000. + 0.1
}
