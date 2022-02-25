use std::time::Duration;

use crate::actions::Actions;
use crate::collide_aabb::collide;
use crate::enemy::{sprite_z, Alive, Corpse, Dead, Enemy};
use crate::loading::{FontAssets, TextureAssets};
use crate::{GameConfiguration, GameState};
use benimator::{Play, SpriteSheetAnimation};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct BaseMoveSpeed(pub f32);

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct HurtBox {
    pub pos: Vec2,
    pub size: Vec2,
}

#[derive(Component)]
pub struct InvincibilityFrames {
    timer: Timer,
}

#[derive(Component)]
struct VFX;

#[derive(Component)]
pub struct AttackNearest {
    pub damage: f32,
    pub interval: Timer,
}

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct XpGem {
    value: u32,
}

#[derive(Component)]
pub struct HitBox {
    pub pos: Vec2,
    pub size: Vec2,
    pub damage: f32,
}

pub struct DealDamageEvent {
    entity: Entity,
}

pub struct DieEvent {
    entity: Entity,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_player)
                .with_system(spawn_camera),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player)
                .with_system(move_camera)
                .with_system(hurt_player)
                .with_system(clean_animation_effects)
                .with_system(trigger_attack_nearest)
                .with_system(deal_damage)
                .with_system(tick_invincibility_frames)
                .with_system(handle_die)
                .with_system(spawn_corpses)
                .with_system(clean_corpses)
                .with_system(gather_xp_gems),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    config: Res<GameConfiguration>,
) {
    /*
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    let texture = asset_server
        .load("textures/heroic-creature-pack 2/HAS Creature Pack 1.2/Castle/CastleSpriteSheet.png");
    let atlas = TextureAtlas::from_grid(texture, Vec2::new(16., 16.), 20, 16);
    let handle = texture_atlases.add(atlas);
    */

    let font = fonts.fira_sans.clone();
    let style = TextStyle {
        font,
        font_size: 10.,
        color: Color::WHITE,
    };
    let alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: textures.castle.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., sprite_z(Vec2::ZERO)))
                .with_scale(Vec3::splat(config.scale)),
            sprite: TextureAtlasSprite::new(141),
            ..Default::default()
        })
        .insert(Player)
        .insert(BaseMoveSpeed(120.))
        .insert(HurtBox {
            pos: Vec2::ZERO,
            size: Vec2::splat(16. * config.scale * 0.6),
        })
        .insert(Health(100.))
        .insert(AttackNearest {
            damage: 1.,
            interval: Timer::from_seconds(2., true),
        })
        .with_children(|parent| {
            parent.spawn_bundle(Text2dBundle {
                text: Text::with_section(&format!("{}", 100.), style, alignment),
                transform: Transform::from_translation(Vec3::new(0., -20., 10.)),
                ..Default::default()
            });
        });
}

fn gather_xp_gems(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut items: Query<(Entity, &mut Transform), (With<XpGem>, With<Item>, Without<Player>)>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    let player = player.single().translation;

    for (e, mut item) in items.iter_mut() {
        let p = &mut item.translation;

        if (player.xy() - p.xy()).length() < 20. {
            println!("Picking up XP!");
            commands.entity(e).despawn_recursive();
            continue;
        }

        if (player.xy() - p.xy()).length() < 150. {
            let dir = (player.xy() - p.xy()).clamp_length_max(1.);
            let SPEED = 500.;
            let new_p = p.xy() + dir * dt * SPEED;

            *p = Vec3::new(new_p.x, new_p.y, sprite_z(new_p));
        }
    }
}

fn clean_animation_effects(
    mut commands: Commands,
    animations: Query<(Entity, &Handle<SpriteSheetAnimation>), (Without<Play>, With<VFX>)>,
) {
    for (e, _animation) in animations.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn deal_damage(
    mut event_deal_damage: EventReader<DealDamageEvent>,
    mut event_die: EventWriter<DieEvent>,
    mut q: Query<&mut Health>,
) {
    for event in event_deal_damage.iter() {
        let mut health = q.get_mut(event.entity).unwrap();
        dbg!(&health.0);
        health.0 -= 1.;

        if health.0 <= 0. {
            event_die.send(DieEvent {
                entity: event.entity,
            });
        }
    }
}

fn clean_corpses(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Corpse)>) {
    for (e, mut corpse) in q.iter_mut() {
        corpse.timer.tick(time.delta());

        if corpse.timer.just_finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn spawn_corpses(
    mut commands: Commands,
    //textures: Res<TextureAssets>,
    mut q: Query<
        (Entity, &Transform, &mut TextureAtlasSprite),
        (With<Enemy>, With<Dead>, Without<Play>),
    >,
) {
    for (e, transform, mut sprite) in q.iter_mut() {
        commands.entity(e).remove::<Dead>().insert(Corpse {
            timer: Timer::from_seconds(5., false),
        });
        sprite.index = 43;
    }
}

fn handle_die(
    mut event_die: EventReader<DieEvent>,
    q: Query<(Entity, &Transform)>,
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    textures: Res<TextureAssets>,
    config: Res<GameConfiguration>,
) {
    let animation_handle = animations.add(
        SpriteSheetAnimation::from_range(
            40..=43,                             // Indices of the sprite atlas
            Duration::from_secs_f64(1.0 / 10.0), // Duration of each frame
        )
        .once(),
    );

    for event in event_die.iter() {
        let entity = event.entity;
        let (entity, transform) = q.get(entity).unwrap();
        let p = transform.translation;

        commands
            .entity(entity)
            .remove::<Alive>()
            .insert(Dead)
            .insert_bundle(SpriteSheetBundle {
                texture_atlas: textures.necromancer.clone(),
                transform: Transform::from_translation(Vec3::new(
                    p.x,
                    p.y,
                    sprite_z(Vec2::new(p.x, p.y)),
                ))
                .with_scale(Vec3::splat(config.scale)),
                ..Default::default()
            })
            .insert(animation_handle.clone())
            .insert(Play);

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures.misc.clone(),
                transform: Transform::from_translation(Vec3::new(
                    p.x,
                    p.y,
                    sprite_z(Vec2::new(p.x, p.y)),
                ))
                .with_scale(Vec3::splat(config.scale / 2.)),
                sprite: TextureAtlasSprite {
                    index: 20,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Item)
            .insert(XpGem { value: 1 });
    }
}

fn trigger_attack_nearest(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, With<Alive>)>,
    mut attacks: Query<&mut AttackNearest>,
    time: Res<Time>,
    mut event_deal_damage: EventWriter<DealDamageEvent>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    textures: Res<TextureAssets>,
    config: Res<GameConfiguration>,
) {
    let player = player.single();
    let mut n_attacks = 0;
    for mut attack in attacks.iter_mut() {
        attack.interval.tick(time.delta());
        if attack.interval.just_finished() {
            n_attacks += 1;
        }
    }
    let mut enemies_distance: Vec<_> = enemies
        .iter()
        .map(|(e, transform)| {
            (
                e,
                (player.translation.xy() - transform.translation.xy()).length(),
                transform,
            )
        })
        .collect();
    enemies_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let targets = enemies_distance.iter().take(n_attacks);

    let animation_handle = animations.add(
        SpriteSheetAnimation::from_range(
            136..=139,                           // Indices of the sprite atlas
            Duration::from_secs_f64(1.0 / 10.0), // Duration of each frame
        )
        .once(),
    );
    for &(entity, _distance, transform) in targets {
        let p = transform.translation;
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures.magic.clone(),
                transform: Transform::from_translation(Vec3::new(
                    p.x,
                    p.y,
                    sprite_z(Vec2::new(p.x, p.y)) + 0.1,
                ))
                .with_scale(Vec3::splat(config.scale)),
                //sprite: TextureAtlasSprite::new(27),
                ..Default::default()
            })
            .insert(animation_handle.clone())
            .insert(Play)
            .insert(VFX);
        event_deal_damage.send(DealDamageEvent { entity });
    }
}

fn tick_invincibility_frames(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut InvincibilityFrames, &mut TextureAtlasSprite)>,
) {
    for (e, mut frame, mut sprite) in q.iter_mut() {
        frame.timer.tick(time.delta());
        if frame.timer.just_finished() {
            commands.entity(e).remove::<InvincibilityFrames>();
            sprite.color = Color::WHITE;
        }
    }
}

fn hurt_player(
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    textures: Res<TextureAssets>,
    mut player: Query<
        (
            Entity,
            &Transform,
            &HurtBox,
            &mut TextureAtlasSprite,
            Option<&InvincibilityFrames>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    enemies: Query<(Entity, &Transform, &HitBox), (With<Enemy>, Without<Player>)>,
    config: Res<GameConfiguration>,
) {
    let (player_entity, player_pos, player_hurt_box, mut sprite, invincibility) =
        player.single_mut();
    if invincibility.is_some() {
        return;
    }

    let mut hit = false;
    for (enemy_entity, enemy_transform, enemy_hit_box) in enemies.iter() {
        if let Some(_collision) = collide(
            enemy_transform.translation + enemy_hit_box.pos.extend(0.),
            enemy_hit_box.size,
            (player_pos.translation.xy() + player_hurt_box.pos).extend(0.),
            player_hurt_box.size,
        ) {
            hit = true;
            dbg!("hit from ", enemy_entity);
            commands.entity(player_entity).insert(InvincibilityFrames {
                timer: Timer::new(Duration::from_secs_f32(0.2), false),
            });

            break;
        }
    }
    if hit {
        sprite.color = Color::RED;
    } else {
        sprite.color = Color::WHITE;
    }
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<
        (&mut Transform, &BaseMoveSpeed, &mut TextureAtlasSprite),
        With<Player>,
    >,
) {
    if actions.player_movement.is_none() {
        return;
    }

    let (mut player_transform, base_speed, mut sprite) = player_query.single_mut();

    let movement =
        actions.player_movement.unwrap().extend(0.) * base_speed.0 * time.delta_seconds();
    if movement.x <= 0. {
        sprite.flip_x = true;
    } else {
        sprite.flip_x = false;
    }
    player_transform.translation += movement;
    player_transform.translation.z = sprite_z(player_transform.translation.xy());
}

fn move_camera(
    mut camera: Query<&mut Transform, (With<Camera>, With<MainCamera>)>,
    player: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera = camera.single_mut();
    let player = player.single();

    camera.translation = player.translation.xy().extend(camera.translation.z);
}
