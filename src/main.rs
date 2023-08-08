use std::ops::RangeInclusive;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct AnimationFrames(RangeInclusive<usize>);

#[derive(Clone, Copy)]
enum Suit {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}

#[derive(Component)]
struct CardData {
    faceup: bool,
    id: usize,
    rank: u8,
    suit: Suit,
}

fn new_card(
    id: usize,
    rank: u8,
    suit: Suit,
    faceup: bool,
    transform: Transform,
    texture_atlas_handle: Handle<TextureAtlas>,
) -> (SpriteSheetBundle, CardData) {
    let data = CardData {
        faceup,
        id,
        rank,
        suit,
    };

    (
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(sprite_index_from_data(&data)),
            transform,
            ..default()
        },
        data,
    )
}

const FACE_DOWN_INDEX: u8 = 52;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn sprite_index_from_data(data: &CardData) -> usize {
    (match data.suit {
        Suit::Hearts => 0,
        Suit::Diamonds => 1,
        Suit::Spades => 2,
        Suit::Clubs => 3,
    } * 13
        + data.rank)
        .into()
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture = asset_server.load("cards.png");
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(34., 48.), 13, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in 0..13u8 {
        for s in 0..4u8 {
            debug!("spawning card ({i},{s})");

            fn trans(i: f32, s: f32, z: f32) -> Vec3 {
                Vec3 {
                    x: 40. * i,
                    y: 54. * s,
                    z,
                }
            }

            commands.spawn(new_card(
                i.into(),
                i % 13,
                [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs][usize::from(s)],
                true,
                Transform::from_translation(trans(i.into(), s.into(), 0.) - trans(6., 1.5, 0.)),
                texture_atlas_handle.clone(),
            ));
        }
    }
}

fn mouse_is_over(mouse_pos: Vec2, card: &Transform) -> bool {
    let size = Vec3 {
        x: 17.,
        y: 24.,
        z: 0.,
    } * card.scale;

    let Vec3 { x, y, .. } = card.translation;

    !(mouse_pos.x < x - size.x
        || mouse_pos.x > x + size.x
        || mouse_pos.y < y - size.y
        || mouse_pos.y > y + size.y)
}

fn flip_card(data: &mut CardData, sprite: &mut TextureAtlasSprite) {
    match data.faceup {
        true => {
            data.faceup = false;
            sprite.index = FACE_DOWN_INDEX.into();
        }
        false => {
            data.faceup = true;
            sprite.index = sprite_index_from_data(data);
        }
    }
}

fn mouse_click(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut transforms: Query<(&mut Transform, &mut CardData, &mut TextureAtlasSprite)>,
) {
    (|| {
        let window = window.single();
        let (camera, camera_transform) = camera.single();

        let mouse_pos = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))?;

        debug!("click at {mouse_pos}");

        let (_, mut data, mut sprite) = transforms
            .iter_mut()
            .filter(|(t, ..)| mouse_is_over(mouse_pos, t))
            .max_by(|(a, ..), (b, ..)| a.translation.z.partial_cmp(&b.translation.z).unwrap())?;

        flip_card(data.as_mut(), sprite.as_mut());

        Some(())
    })();
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            mouse_click.run_if(input_just_pressed(MouseButton::Left)),
        )
        .run();
}
