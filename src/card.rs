use bevy::prelude::{Component, Vec2};

#[derive(Clone, Copy)]
pub enum Suit {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}

pub type CardID = usize;

#[derive(Component)]
pub struct Card {
    pub faceup: bool,
    pub id: CardID,
    pub rank: u8,
    pub suit: Suit,
    pub picked_up_offset: Option<Vec2>,
}

const FACE_DOWN_INDEX: u8 = 52;

impl Card {
    pub fn sprite_index(&self) -> usize {
        if self.faceup {
            (match self.suit {
                Suit::Hearts => 0,
                Suit::Diamonds => 1,
                Suit::Spades => 2,
                Suit::Clubs => 3,
            } * 13
                + self.rank)
                .into()
        } else {
            FACE_DOWN_INDEX.into()
        }
    }
}
