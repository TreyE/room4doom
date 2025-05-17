//! A custom statusbar to show the players status during gameplay.
//!
//! Displays things like ammo count, weapons owned, key/skulls owned, health and
//! so on.

use faces::DoomguyFace;
use gamestate_traits::util::{
    draw_num_pixels, draw_percent_pixels, get_large_percent_sprites, get_num_sprites,
    get_small_percent_sprites, get_st_key_sprites,
};
use gamestate_traits::{
    AmmoType, GameMode, GameTraits, PixelBuffer, PlayerStatus, Scancode, SubsystemTrait,
    WEAPON_INFO, WeaponType,
};
use std::collections::HashMap;
use wad::WadData;
use wad::types::{WadPalette, WadPatch};

mod faces;

pub struct Statusbar {
    screen_width: i32,
    screen_height: i32,
    status_top: i32,
    status_left: i32,
    status_right: i32,
    status_bottom: i32,
    status_width: i32,
    mode: GameMode,
    palette: WadPalette,
    background: WadPatch,
    patches: HashMap<&'static str, WadPatch>,
    /// Nums, index is the actual number
    big_nums: [WadPatch; 11],
    lil_nums: [WadPatch; 11],
    grey_nums: [WadPatch; 10],
    yell_nums: [WadPatch; 10],
    /// Keys: blue yellow red. Skulls: blue yellow red
    keys: [WadPatch; 6],
    status: PlayerStatus,
    faces: DoomguyFace,
}

impl Statusbar {
    pub fn new(mode: GameMode, wad: &WadData) -> Self {
        let palette = wad.playpal_iter().next().unwrap();

        let mut patches = HashMap::new();

        let lump = wad.get_lump("STFB1").unwrap();
        patches.insert("STFB1", WadPatch::from_lump(lump));

        Self {
            screen_width: 0,
            screen_height: 0,
            status_top: 0,
            status_left: 0,
            status_right: 0,
            status_width: 0,
            status_bottom: 0,
            mode,
            palette,
            patches,
            background: WadPatch::from_lump(wad.get_lump("STBAR").unwrap()),
            big_nums: get_large_percent_sprites(wad),
            lil_nums: get_small_percent_sprites(wad),
            grey_nums: get_num_sprites("STGNUM", 0, wad),
            yell_nums: get_num_sprites("STYSNUM", 0, wad),
            keys: get_st_key_sprites(wad),
            status: PlayerStatus::default(),
            faces: DoomguyFace::new(wad),
        }
    }

    fn get_patch(&self, name: &str) -> &WadPatch {
        self.patches
            .get(name)
            .unwrap_or_else(|| panic!("{name} not in cache"))
    }

    fn draw_health_pixels(&self, big: bool, face: bool, pixels: &mut impl PixelBuffer) {
        let nums = if big { &self.big_nums } else { &self.lil_nums };

        let mut y = nums[0].height as i32;
        let mut x = self.status_left + (nums[0].width as f32 * 3.4).floor() as i32;
        if !big {
            y = y * 2 + 2;
            x *= 5;
        } else {
            y = y + self.lil_nums[0].height as i32 + 1;
            //x *= 4;
        }

        let h = if self.status.health < 0 {
            0
        } else {
            self.status.health as u32
        };

        if h < 100 {
            x += nums[0].width as i32;
        }
        if h < 10 {
            x += nums[0].width as i32;
        }
        draw_percent_pixels(h, x, self.status_bottom - 2 - y, nums, self, pixels);
    }

    fn draw_armour_pixels(&self, face: bool, pixels: &mut impl PixelBuffer) {
        if self.status.armorpoints < 0 {
            return;
        }

        let nums = &self.big_nums;

        let mut y = nums[0].height as i32;
        let mut x = nums[0].width as i32;

        y = y + self.lil_nums[0].height as i32 + 1;
        if face {
            x = self.status_left
                + (self.status_width / 2)
                + (nums[0].width as f32 * 1.4).floor() as i32;
        }

        let h = self.status.armorpoints as u32;
        if h < 100 {
            x += nums[0].width as i32;
        }
        if h < 10 {
            x += nums[0].width as i32;
        }
        draw_percent_pixels(h, x, self.status_bottom - 2 - y, nums, self, pixels);
    }

    fn draw_ammo_big_pixels(&self, pixels: &mut impl PixelBuffer) {
        if matches!(self.status.readyweapon, WeaponType::NoChange) {
            return;
        }
        if !(self.mode == GameMode::Commercial)
            && self.status.readyweapon == WeaponType::SuperShotgun
        {
            return;
        }

        let ammo = WEAPON_INFO[self.status.readyweapon as usize].ammo;
        if ammo == AmmoType::NoAmmo {
            return;
        }

        let height = self.big_nums[0].height as i32 + self.lil_nums[0].height as i32 + 1;
        let mut start_x = self.status_left as i32;
        let ammo = self.status.ammo[ammo as usize];
        if ammo < 100 {
            start_x += self.big_nums[0].width as i32;
        }
        if ammo < 10 {
            start_x += self.big_nums[0].width as i32;
        }

        draw_num_pixels(
            ammo,
            start_x,
            self.status_bottom - 2 - (height as i32),
            0,
            &self.big_nums,
            self,
            pixels,
        );
    }

    fn draw_keys_pixels(&self, pixels: &mut impl PixelBuffer) {
        let height = self.keys[3].height as i32;
        let width = self.keys[0].width as i32;

        let skull_x = self.screen_width - width - 4;
        let mut x = skull_x - width - 2;
        let start_y = self.screen_height - height - 2;

        for (mut i, owned) in self.status.cards.iter().enumerate() {
            if !*owned {
                continue;
            }

            let height = self.keys[3].height as i32;
            let patch = &self.keys[i];
            let mut pad = 0;
            if i > 2 {
                i -= 3;
                x = skull_x;
            } else {
                pad = -3;
            }
            self.draw_patch_pixels(
                patch,
                x,
                start_y - pad - height * i as i32 - i as i32,
                pixels,
            );
        }
    }

    fn draw_weapons_pixels(&self, pixels: &mut impl PixelBuffer) {
        let y = self.grey_nums[0].height as i32;
        let x = self.grey_nums[0].width as i32;
        let mult = if self.mode == GameMode::Commercial {
            10
        } else {
            9
        };
        let start_x = self.screen_width
            - self.grey_nums[0].width as i32 * mult // align with big ammo
            - self.big_nums[0].width as i32
            - self.keys[0].width as i32 - 2;
        let start_y = self.screen_height - y - 2;

        for (i, owned) in self.status.weaponowned.iter().enumerate() {
            if !(self.mode == GameMode::Commercial) && i == 8 || !*owned {
                continue;
            }
            let nums = if self.status.readyweapon as usize == i {
                &self.yell_nums
            } else {
                &self.grey_nums
            };
            draw_num_pixels(
                i as u32 + 1,
                start_x + x * i as i32 + i as i32,
                start_y,
                0,
                nums,
                self,
                pixels,
            );
        }
    }

    fn draw_face_pixels(&self, mut big: bool, upper: bool, pixels: &mut impl PixelBuffer) {
        if upper {
            big = true;
        }

        let mut x;
        let mut y;
        if big && !upper {
            let patch = self.get_patch("STFB1");
            y = if upper {
                0
            } else {
                self.status_bottom - patch.height as i32
            };
            x = self.screen_width / 2 - patch.width as i32 / 2;
            self.draw_patch_pixels(patch, x, y, pixels);
        };

        let patch = self.faces.get_face();
        let offset_y = patch.height as i32;
        x = self.status_left + (self.status_width / 2) - ((patch.width as i32) / 2)
            + (patch.left_offset as i32);
        y = self.status_bottom - offset_y;
        self.draw_patch_pixels(patch, x, y, pixels);
    }
}

impl SubsystemTrait for Statusbar {
    fn init(&mut self, _game: &impl GameTraits) {}

    fn responder(&mut self, _sc: Scancode, _game: &mut impl GameTraits) -> bool {
        false
    }

    fn ticker(&mut self, game: &mut impl GameTraits) -> bool {
        self.status = game.player_status();
        self.faces.tick(&self.status);
        false
    }

    fn get_palette(&self) -> &WadPalette {
        &self.palette
    }

    fn draw(&mut self, buffer: &mut impl PixelBuffer) {
        self.screen_width = 320;
        self.screen_height = 200;
        self.status_top = self.screen_height - self.background.height as i32;
        self.status_left = self.background.left_offset as i32;
        self.status_right = self.status_left + self.background.width as i32;
        self.status_bottom = self.status_top + self.background.height as i32;
        self.status_width = self.background.width as i32;
        self.draw_patch_pixels(&self.background, self.status_left, self.status_top, buffer);

        let face = true;
        if face {
            self.draw_face_pixels(false, false, buffer);
        }
        self.draw_health_pixels(true, face, buffer);
        self.draw_armour_pixels(face, buffer);
        self.draw_ammo_big_pixels(buffer);
        self.draw_weapons_pixels(buffer);
        self.draw_keys_pixels(buffer);
    }
}
