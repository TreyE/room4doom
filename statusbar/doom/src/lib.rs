//! A custom statusbar to show the players status during gameplay.
//!
//! Displays things like ammo count, weapons owned, key/skulls owned, health and
//! so on.

use faces::DoomguyFace;
use gamestate_traits::util::{draw_num_pixels, get_num_sprites, get_st_key_sprites};
use gamestate_traits::{
    AmmoType, GameMode, GameTraits, PixelBuffer, PlayerStatus, Scancode, SubsystemTrait,
    WEAPON_INFO, WeaponType,
};
use std::collections::HashMap;
use wad::WadData;
use wad::types::{WadPalette, WadPatch};

mod classic;
mod custom;
mod faces;

pub use classic::ClassicStatusbar;
pub use custom::CustomStatusbar;
