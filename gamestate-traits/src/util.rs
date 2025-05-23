use crate::SubsystemTrait;
use render_trait::PixelBuffer;
use std::mem::MaybeUninit;
use wad::WadData;
use wad::types::{WAD_PATCH, WadPatch};

/// Pattern like `WINUM` or `STTNUM`
pub fn get_num_sprites(pattern: &str, start: usize, wad: &WadData) -> [WadPatch; 10] {
    let mut nums: [WadPatch; 10] = [WAD_PATCH; 10];
    for (i, num) in nums.iter_mut().enumerate() {
        let p = i + start;
        let lump = wad.get_lump(&format!("{pattern}{p}")).unwrap();
        *num = WadPatch::from_lump(lump);
    }
    nums
}

pub fn get_small_percent_sprites(wad: &WadData) -> [WadPatch; 11] {
    let start = 48;
    let mut nums: [WadPatch; 11] = [WAD_PATCH; 11];
    for (i, num) in nums.iter_mut().enumerate() {
        let p = i + start;
        if i == 10 {
            let lump = wad.get_lump(&format!("STCFN037")).unwrap();
            *num = WadPatch::from_lump(lump);
        } else {
            let lump = wad.get_lump(&format!("STCFN0{p}")).unwrap();
            *num = WadPatch::from_lump(lump);
        }
    }
    nums
}

pub fn get_large_percent_sprites(wad: &WadData) -> [WadPatch; 11] {
    let mut nums: [WadPatch; 11] = [WAD_PATCH; 11];
    for (i, num) in nums.iter_mut().enumerate() {
        let p = i;
        if i == 10 {
            let lump = wad.get_lump(&format!("STTPRCNT")).unwrap();
            *num = WadPatch::from_lump(lump);
        } else {
            let lump = wad.get_lump(&format!("STTNUM{p}")).unwrap();
            *num = WadPatch::from_lump(lump);
        }
    }
    nums
}

pub fn get_st_key_sprites(wad: &WadData) -> [WadPatch; 6] {
    let mut keys: [MaybeUninit<WadPatch>; 6] = [
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
    ];
    for (i, key) in keys.iter_mut().enumerate() {
        let lump = wad.get_lump(&format!("STKEYS{i}")).unwrap();
        *key = MaybeUninit::new(WadPatch::from_lump(lump));
    }
    unsafe { keys.map(|n| n.assume_init()) }
}

pub fn draw_percent_pixels(
    p: u32,
    mut x: i32,
    y: i32,
    nums: &[WadPatch],
    drawer: &impl SubsystemTrait,
    pixels: &mut impl PixelBuffer,
) -> i32 {
    let width = nums[0].width as i32;
    let digits: Vec<u32> = p
        .to_string()
        .chars()
        .map(|d| d.to_digit(10).unwrap())
        .collect();

    for n in digits.iter() {
        let num = &nums[*n as usize];
        drawer.draw_patch_pixels(num, x, y, pixels);
        x += width;
    }
    drawer.draw_patch_pixels(&nums[10 as usize], x, y, pixels);
    x += width;
    x
}

pub fn draw_num_pixels(
    p: u32,
    mut x: i32,
    y: i32,
    pad: usize,
    nums: &[WadPatch],
    drawer: &impl SubsystemTrait,
    pixels: &mut impl PixelBuffer,
) -> i32 {
    let width = nums[0].width as i32;
    let digits: Vec<u32> = p
        .to_string()
        .chars()
        .map(|d| d.to_digit(10).unwrap())
        .collect();

    for n in digits.iter() {
        let num = &nums[*n as usize];
        drawer.draw_patch_pixels(num, x, y, pixels);
        x += width;
    }
    if digits.len() <= pad {
        for _ in 0..=pad - digits.len() {
            x -= width;
            drawer.draw_patch_pixels(&nums[0], x, y, pixels);
        }
    }

    x
}

pub fn draw_num(
    p: u32,
    x: i32,
    y: i32,
    pad: usize,
    nums: &[WadPatch],
    drawer: &impl SubsystemTrait,
    buffer: &mut impl PixelBuffer,
) -> i32 {
    // TODO: remove duplicated functionality
    draw_num_pixels(p, x, y, pad, nums, drawer, buffer)
}
