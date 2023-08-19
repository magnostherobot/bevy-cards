use std::cmp::Ordering::Equal;

use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_just_released},
        mouse::MouseMotion,
    },
    prelude::*,
    window::PrimaryWindow,
};

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
    id: CardID,
    rank: u8,
    suit: Suit,
    picked_up_offset: Option<Vec2>,
}

#[derive(Component, Deref, DerefMut)]
struct ZIndex(f32);

type CardID = usize;

#[derive(Event)]
struct CardFlip(CardID);

#[derive(Event)]
struct CardPickUp(CardID, Vec2);

#[derive(Event)]
struct CardPutDown;

fn new_card(
    id: CardID,
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
        picked_up_offset: None,
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

fn card_grid(texture_atlas_handle: &Handle<TextureAtlas>) -> Vec<(SpriteSheetBundle, CardData)> {
    (0..13u8)
        .flat_map(|i| {
            (0..4u8).map(move |s| {
                fn trans(i: f32, s: f32, z: f32) -> Vec3 {
                    Vec3 {
                        x: 40. * i,
                        y: 54. * s,
                        z,
                    }
                }

                new_card(
                    (s * 13 + i).into(),
                    i % 13,
                    [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs][usize::from(s)],
                    true,
                    Transform::from_translation(trans(i.into(), s.into(), 0.) - trans(6., 1.5, 0.)),
                    texture_atlas_handle.clone(),
                )
            })
        })
        .collect()
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture = asset_server.load("cards.png");
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(34., 48.), 13, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let cards = card_grid(&texture_atlas_handle);
    let z_index = ZIndex(
        cards
            .iter()
            .map(|c| c.0.transform.translation.z)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
            .unwrap(),
    );

    commands.spawn(z_index);
    commands.spawn_batch(cards);
}

fn mouse_is_over(mouse_pos: Vec2, card: &Transform) -> bool {
    let size = Vec2 { x: 17., y: 24. } * card.scale.truncate();

    let Vec3 { x, y, .. } = card.translation;

    !(mouse_pos.x < x - size.x
        || mouse_pos.x > x + size.x
        || mouse_pos.y < y - size.y
        || mouse_pos.y > y + size.y)
}

fn flip_cards(
    mut cards: Query<(&mut CardData, &mut TextureAtlasSprite)>,
    mut events: EventReader<CardFlip>,
) {
    fn flip_card(data: &mut CardData, sprite: &mut TextureAtlasSprite) {
        if data.faceup {
            data.faceup = false;
            sprite.index = FACE_DOWN_INDEX.into();
        } else {
            data.faceup = true;
            sprite.index = sprite_index_from_data(data);
        }
    }

    for CardFlip(id) in events.iter() {
        for (mut data, mut sprite) in cards.iter_mut() {
            if data.id == *id {
                flip_card(data.as_mut(), sprite.as_mut());
            }
        }
    }
}

fn pick_up_card(
    mut cards: Query<(&mut CardData, &mut Transform)>,
    mut z_index: Query<&mut ZIndex>,
    mut events: EventReader<CardPickUp>,
) {
    for CardPickUp(id, offset) in events.iter() {
        for (mut data, mut transform) in cards.iter_mut().filter(|(c, _)| c.id == *id) {
            data.picked_up_offset = Some(*offset);

            let mut z_index = z_index.single_mut();
            **z_index += 1.;
            transform.translation.z = **z_index;
        }
    }
}

fn put_down_card(mut cards: Query<&mut CardData>, mut events: EventReader<CardPutDown>) {
    if !events.is_empty() {
        events.clear();

        for mut data in cards.iter_mut().filter(|c| c.picked_up_offset.is_some()) {
            data.picked_up_offset = None;
        }
    }
}

fn move_picked_up_cards(
    mut cards: Query<(&mut CardData, &mut Transform)>,
    mut motions: EventReader<MouseMotion>,
) {
    for MouseMotion { delta } in motions.iter() {
        for (_, mut transform) in cards.iter_mut().filter(|c| c.0.picked_up_offset.is_some()) {
            transform.translation += Vec3 {
                x: delta.x,
                y: -delta.y,
                z: 0.,
            };
        }
    }
}

fn mouse_click(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    transforms: Query<(&Transform, &CardData)>,
    mut events: EventWriter<CardPickUp>,
) {
    (|| {
        let window = window.single();
        let (camera, camera_transform) = camera.single();

        let mouse_pos = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))?;

        debug!("click at {mouse_pos}");

        let (transform, data) = transforms
            .iter()
            .filter(|(t, ..)| mouse_is_over(mouse_pos, t))
            .max_by(|(a, ..), (b, ..)| a.translation.z.partial_cmp(&b.translation.z).unwrap())?;

        events.send(CardPickUp(
            data.id,
            mouse_pos - transform.translation.truncate(),
        ));

        Some(())
    })();
}

fn mouse_release(mut events: EventWriter<CardPutDown>) {
    events.send(CardPutDown);
}

fn main() {
    App::new()
        .add_event::<CardFlip>()
        .add_event::<CardPickUp>()
        .add_event::<CardPutDown>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            mouse_click.run_if(input_just_pressed(MouseButton::Left)),
        )
        .add_systems(
            Update,
            mouse_release.run_if(input_just_released(MouseButton::Left)),
        )
        .add_systems(Update, flip_cards.after(mouse_click))
        .add_systems(Update, pick_up_card.after(mouse_click))
        .add_systems(Update, put_down_card)
        .add_systems(Update, move_picked_up_cards)
        .run();
}
