//! Doom source name `p_pspr`

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use sound_traits::SfxName;

use crate::doom_def::{MELEERANGE, MISSILERANGE, PowerType, WEAPON_INFO};
use crate::info::{STATES, State, StateNum};
use crate::player::{Player, PsprNum};
use crate::thing::MapObject;
use crate::tic_cmd::TIC_CMD_BUTTONS;
use crate::{MapObjKind, PlayerState, WeaponType};
use math::{ANG45, ANG90, ANG180, Angle, FT_FOUR, FT_ONE, fixed_t, p_random, point_to_angle_2};

const LOWERSPEED: fixed_t = fixed_t::from_int(6);
const RAISESPEED: fixed_t = fixed_t::from_int(6);
pub(crate) const WEAPONBOTTOM: fixed_t = fixed_t::from_int(128);
const WEAPONTOP: fixed_t = fixed_t::from_int(32);

/// From P_PSPR
#[derive(Debug)]
pub struct PspDef {
    /// a NULL state means not active
    pub state: Option<&'static State>,
    pub(crate) tics: i32,
    pub sx: fixed_t,
    pub sy: fixed_t,
}

/// The player can re-fire the weapon
/// without lowering it entirely.
pub(crate) fn a_refire(player: &mut Player, _pspr: &mut PspDef) {
    if player.cmd.buttons & TIC_CMD_BUTTONS.bt_attack != 0
        && player.pendingweapon == WeaponType::NoChange
        && player.status.health != 0
    {
        player.refire += 1;
        player.fire_weapon();
    } else {
        player.refire = 0;
        player.check_ammo();
    }
}

pub(crate) fn a_weaponready(player: &mut Player, pspr: &mut PspDef) {
    let mut level_time = 0;
    let readyweapon = player.status.readyweapon;
    if let Some(mobj) = player.mobj_mut() {
        if std::ptr::eq(mobj.state, unsafe { &STATES[StateNum::PLAY_ATK1 as usize] })
            || std::ptr::eq(mobj.state, unsafe { &STATES[StateNum::PLAY_ATK2 as usize] })
        {
            mobj.set_state(StateNum::PLAY);
        }

        level_time = unsafe { (*mobj.level).level_time };

        if let Some(state) = pspr.state {
            let check = unsafe { &STATES[StateNum::SAW as usize] };
            if readyweapon == WeaponType::Chainsaw
                && state.sprite == check.sprite
                && state.frame == check.frame
                && state.next_state == check.next_state
            {
                mobj.start_sound(SfxName::Sawidl);
            }
        }
    }

    // check for change
    //  if player is dead, put the weapon away
    if player.pendingweapon != WeaponType::NoChange || player.status.health <= 0 {
        // change weapon
        //  (pending weapon should allready be validated)
        if player.status.readyweapon != WeaponType::NoChange {
            let new_state = WEAPON_INFO[player.status.readyweapon as usize].downstate;
            player.set_psprite(PsprNum::Weapon as usize, new_state);
        }
        return;
    }

    //  the missile launcher and bfg do not auto fire
    if player.cmd.buttons & TIC_CMD_BUTTONS.bt_attack != 0 {
        if !player.status.attackdown
            || (player.status.readyweapon != WeaponType::Missile
                && player.status.readyweapon != WeaponType::BFG)
        {
            player.status.attackdown = true;
            player.fire_weapon();
            return;
        }
    } else {
        player.status.attackdown = false;
    }

    // the division is the frequency
    let angle = (level_time as f32 / 8.0).cos();
    pspr.sx = player.bob * fixed_t::from_float(angle);
    // the division is the frequency
    let angle = (level_time as f32 / 4.0).sin();
    // the division (3.0) is the depth
    pspr.sy = WEAPONTOP
        + fixed_t::from_int(6)
        + player.bob / fixed_t::from_int(3) * fixed_t::from_float(angle);
}

pub(crate) fn a_lower(player: &mut Player, pspr: &mut PspDef) {
    pspr.sy += LOWERSPEED;
    if pspr.sy < WEAPONBOTTOM {
        return;
    }

    if player.player_state == PlayerState::Dead {
        // Keep weapon down if dead
        pspr.sy = WEAPONBOTTOM;
        return;
    }

    if player.status.health <= 0 {
        // Player died so take weapon off screen
        player.set_psprite(PsprNum::Weapon as usize, StateNum::None);
        return;
    }

    player.status.readyweapon = player.pendingweapon;
    player.bring_up_weapon();
}

pub(crate) fn a_raise(player: &mut Player, pspr: &mut PspDef) {
    pspr.sy -= RAISESPEED;
    if pspr.sy > WEAPONTOP {
        return;
    }
    pspr.sy = WEAPONTOP;

    let new_state = WEAPON_INFO[player.status.readyweapon as usize].readystate;
    player.set_psprite(PsprNum::Weapon as usize, new_state);
}

fn shoot_bullet(player: &mut Player) {
    let distance = MISSILERANGE;
    let refire = player.refire;
    if let Some(mobj) = player.mobj_mut() {
        mobj.start_sound(SfxName::Pistol);
        mobj.set_state(StateNum::PLAY_ATK2);

        let mut bsp_trace = mobj.get_shoot_bsp_trace(distance);
        let bullet_slope = mobj.bullet_slope(distance, &mut bsp_trace);
        mobj.gun_shot(refire == 0, distance, bullet_slope, &mut bsp_trace);
    }
}

pub(crate) fn a_firepistol(player: &mut Player, _pspr: &mut PspDef) {
    shoot_bullet(player);
    player.status.ammo[WEAPON_INFO[player.status.readyweapon as usize].ammo as usize] -= 1;
    player.set_psprite(
        PsprNum::Flash as usize,
        WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    );
}

pub(crate) fn a_fireshotgun(player: &mut Player, _pspr: &mut PspDef) {
    let distance = MISSILERANGE;

    if let Some(mobj) = player.mobj_mut() {
        mobj.start_sound(SfxName::Shotgn);
        mobj.set_state(StateNum::PLAY_ATK2);

        let mut bsp_trace = mobj.get_shoot_bsp_trace(distance);
        let bullet_slope = mobj.bullet_slope(distance, &mut bsp_trace);

        for _ in 0..7 {
            mobj.gun_shot(false, distance, bullet_slope.clone(), &mut bsp_trace);
        }
    }

    player.subtract_readyweapon_ammo(1);
    player.set_psprite(
        PsprNum::Flash as usize,
        WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    );
}

pub(crate) fn a_fireshotgun2(player: &mut Player, _pspr: &mut PspDef) {
    let distance = MISSILERANGE;

    if let Some(mobj) = player.mobj_mut() {
        mobj.start_sound(SfxName::Dshtgn);
        mobj.set_state(StateNum::PLAY_ATK2);

        let mut bsp_trace = mobj.get_shoot_bsp_trace(distance);
        let bullet_slope = mobj.bullet_slope(distance, &mut bsp_trace);

        for _ in 0..20 {
            let damage = (p_random() % 3 + 1) * 5;
            let mut angle = mobj.angle;
            angle += Angle::from_int((p_random() - p_random()) << 19);
            mobj.line_attack(
                damage,
                MISSILERANGE,
                angle,
                bullet_slope.clone(),
                &mut bsp_trace,
            );
        }
    }

    player.subtract_readyweapon_ammo(2);
    player.set_psprite(
        PsprNum::Flash as usize,
        WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    );
}

pub(crate) fn a_firecgun(player: &mut Player, pspr: &mut PspDef) {
    if !player.check_ammo() {
        return;
    }
    shoot_bullet(player);
    let state = StateNum::from(
        WEAPON_INFO[player.status.readyweapon as usize].flashstate as u16
            + pspr.state.unwrap().next_state as u16
            - StateNum::CHAIN1 as u16
            - 1,
    );

    player.subtract_readyweapon_ammo(1);
    player.set_psprite(PsprNum::Flash as usize, state);
}

pub(crate) fn a_fireplasma(player: &mut Player, _pspr: &mut PspDef) {
    player.subtract_readyweapon_ammo(1);
    let state = StateNum::from(
        (WEAPON_INFO[player.status.readyweapon as usize].flashstate as u16 + p_random() as u16) & 1,
    );
    player.set_psprite(PsprNum::Flash as usize, state);
    if let Some(mobj) = player.mobj_raw() {
        unsafe {
            (*mobj).start_sound(SfxName::Plasma);
            MapObject::spawn_player_missile(
                &mut *mobj,
                crate::MapObjKind::MT_PLASMA,
                &mut (*(*mobj).level),
            );
        }
    }
}

pub(crate) fn a_firemissile(player: &mut Player, _pspr: &mut PspDef) {
    player.subtract_readyweapon_ammo(1);
    // player.set_psprite(
    //     PsprNum::Flash as usize,
    //     WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    // );
    if let Some(mobj) = player.mobj_raw() {
        unsafe {
            (*mobj).start_sound(SfxName::Rlaunc);
            MapObject::spawn_player_missile(
                &mut *mobj,
                crate::MapObjKind::MT_ROCKET,
                &mut (*(*mobj).level),
            );
        }
    }
}

pub(crate) fn a_firebfg(player: &mut Player, _pspr: &mut PspDef) {
    player.subtract_readyweapon_ammo(1);
    // player.set_psprite(
    //     PsprNum::Flash as usize,
    //     WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    // );
    if let Some(mobj) = player.mobj_raw() {
        unsafe {
            MapObject::spawn_player_missile(
                &mut *mobj,
                crate::MapObjKind::MT_BFG,
                &mut (*(*mobj).level),
            );
        }
    }
}

pub(crate) fn a_bfgsound(player: &mut Player, _pspr: &mut PspDef) {
    player.start_sound(SfxName::Bfg);
}

pub(crate) fn a_bfgspray(player: &mut MapObject) {
    for i in 0..40 {
        // From left to right
        let angle = player.angle - Angle::new(ANG45) + Angle::new(ANG90 / 40 * i);
        //FRAC_PI_4 + (FRAC_PI_2 / 40.0) * i as f32;
        let mut bsp_trace = player.get_shoot_bsp_trace(MISSILERANGE);
        let old_angle = player.angle;
        player.angle = angle;
        let aim = player.aim_line_attack(MISSILERANGE, &mut bsp_trace);
        player.angle = old_angle;
        if let Some(aim) = aim {
            let mut lt = aim.line_target;
            let level = unsafe { &mut *player.level };
            let z = lt.z + (lt.height / FT_FOUR);
            MapObject::spawn_map_object(lt.xy.x, lt.xy.y, z, MapObjKind::MT_EXTRABFG, level);

            let mut damage = 0;
            for _ in 0..15 {
                damage += (p_random() & 7) + 1;
            }
            lt.p_take_damage(Some(player), None, false, damage);
        }
    }
}

pub(crate) fn a_gunflash(player: &mut Player, _pspr: &mut PspDef) {
    player.set_mobj_state(StateNum::PLAY_ATK2);
    player.set_psprite(
        PsprNum::Flash as usize,
        WEAPON_INFO[player.status.readyweapon as usize].flashstate,
    );
}

pub(crate) fn a_punch(player: &mut Player, _pspr: &mut PspDef) {
    let mut damage = (p_random() % 10 + 1) << 1;
    if player.status.powers[PowerType::Strength as usize] != 0 {
        damage *= 10;
    }

    if let Some(mobj) = player.mobj_mut() {
        let mut angle = mobj.angle;
        angle += Angle::new(((p_random() - p_random()) as u32) << 18);

        //(((p_random() - p_random())) as f32).to_radians();

        let mut bsp_trace = mobj.get_shoot_bsp_trace(MELEERANGE);
        let slope = mobj.aim_line_attack(MELEERANGE, &mut bsp_trace);
        mobj.line_attack(damage, MELEERANGE, angle, slope.clone(), &mut bsp_trace);

        if let Some(res) = slope {
            let target = res.line_target;
            mobj.start_sound(SfxName::Punch);
            mobj.angle = point_to_angle_2(mobj.xy, target.xy);
        }
    }
}

pub(crate) fn a_checkreload(player: &mut Player, _pspr: &mut PspDef) {
    player.check_ammo();
}

pub(crate) fn a_openshotgun2(player: &mut Player, _pspr: &mut PspDef) {
    player.start_sound(SfxName::Dbopn);
}

pub(crate) fn a_loadshotgun2(player: &mut Player, _pspr: &mut PspDef) {
    player.start_sound(SfxName::Dbload);
}

pub(crate) fn a_closeshotgun2(player: &mut Player, pspr: &mut PspDef) {
    player.start_sound(SfxName::Dbcls);
    a_refire(player, pspr);
}

pub(crate) fn a_saw(player: &mut Player, _pspr: &mut PspDef) {
    let damage = 2 * (p_random() % 10 + 1);

    if let Some(mobj) = player.mobj_mut() {
        let mut angle = mobj.angle;
        angle += Angle::new(((p_random() - p_random()) as u32) << 18);

        let mut bsp_trace = mobj.get_shoot_bsp_trace(MELEERANGE + FT_ONE);
        let slope = mobj.aim_line_attack(MELEERANGE + FT_ONE, &mut bsp_trace);
        mobj.line_attack(
            damage,
            MELEERANGE + FT_ONE,
            angle,
            slope.clone(),
            &mut bsp_trace,
        );

        if slope.is_none() {
            mobj.start_sound(SfxName::Sawful);
            return;
        }

        // Have a target
        mobj.start_sound(SfxName::Sawhit);
        if let Some(res) = slope {
            let target = res.line_target;
            mobj.start_sound(SfxName::Punch);
            let angle = point_to_angle_2(mobj.xy, target.xy);

            let delta = angle - mobj.angle;
            if delta > Angle::new(ANG180) {
                if delta < Angle::new((0 as u32).wrapping_sub(ANG90 / 20)) {
                    mobj.angle = angle + Angle::new(ANG90 / 20);
                } else {
                    mobj.angle -= Angle::new(ANG90 / 20);
                }
            } else {
                if (angle - mobj.angle > Angle::new(ANG90 / 20)) {
                    mobj.angle = angle - Angle::new(ANG90 / 21)
                } else {
                    mobj.angle -= Angle::new(ANG90 / 20);
                }
            }
        }
    }
}

pub(crate) fn a_light0(player: &mut Player, _pspr: &mut PspDef) {
    player.extralight = 0;
}

pub(crate) fn a_light1(player: &mut Player, _pspr: &mut PspDef) {
    player.extralight = 1;
}

pub(crate) fn a_light2(player: &mut Player, _pspr: &mut PspDef) {
    player.extralight = 2;
}
