use log::info;
use math::{
    Angle, FT_MAX, FT_MIN, FT_ONE, FT_TWO, FT_ZERO, Trace, VecF2, fixed_t, intercept_vector,
    point_on_side,
};

use crate::{
    Level, LineDefFlags, MapObject, MapPtr,
    level::map_defs::{Blockmap, LineDef},
    thinker::Thinker,
};

#[derive(Debug)]
pub struct AimTrace {
    pub(crate) start: VecF2,
    pub(crate) end: VecF2,
    pub(crate) min_aim_slope: fixed_t,
    pub(crate) max_aim_slope: fixed_t,
    pub(crate) distance: fixed_t,
    pub(crate) angle: Angle,
    pub(crate) shoot_z: fixed_t,
}

#[derive(Default, Clone)]
pub(crate) struct Intercept {
    pub frac: fixed_t,
    pub line: Option<MapPtr<LineDef>>,
    pub thing: Option<*mut Thinker>,
}

pub(crate) struct TraverseLimits {
    max_slope: fixed_t,
    min_slope: fixed_t,
}

#[derive(Debug)]
pub(crate) struct AimHit {
    thing: *mut Thinker,
    pub(crate) slope: fixed_t,
    hit_location: VecF2,
    hit_z: fixed_t,
}

pub(crate) struct ShotHit {
    thing: Option<*mut Thinker>,
    line: Option<MapPtr<LineDef>>,
    hit_location: VecF2,
    hit_z: fixed_t,
}

impl AimTrace {
    pub(crate) fn from_origin(
        x: fixed_t,
        y: fixed_t,
        min_aim: fixed_t,
        max_aim: fixed_t,
        angle: Angle,
        distance: fixed_t,
        shoot_z: fixed_t,
    ) -> Self {
        let end_location_x = x + distance * angle.finecos();
        let end_location_y = y + distance * angle.finesin();
        AimTrace {
            start: VecF2::new(x, y),
            distance,
            angle,
            end: VecF2::new(end_location_x, end_location_y),
            shoot_z,
            min_aim_slope: min_aim,
            max_aim_slope: max_aim,
        }
    }

    // Provide a list of indexes to walk for the blockmap
    fn block_march(&self, blockmap: &Blockmap) -> Vec<(usize, usize)> {
        let mut result = Vec::new();

        let xstart = self.start.x - blockmap.x_origin;
        let xend = self.end.x - blockmap.x_origin;

        let ystart = self.start.y - blockmap.y_origin;
        let yend = self.end.y - blockmap.y_origin;

        let mut minx;
        let mut miny;
        let mut maxx;
        let mut maxy;

        if (xend >= xstart) {
            minx = xstart >> 23;
            maxx = xend >> 23;
        } else {
            minx = xend >> 23;
            maxx = xstart >> 23;
        }

        if (yend >= ystart) {
            miny = ystart >> 23;
            maxy = yend >> 23;
        } else {
            miny = yend >> 23;
            maxy = ystart >> 23;
        }

        if minx < FT_ZERO {
            minx = FT_ZERO;
        }
        if miny < FT_ZERO {
            miny = FT_ZERO;
        }

        maxx = maxx + fixed_t::new(1);
        maxy = maxy + fixed_t::new(1);

        if maxx > fixed_t::new(blockmap.columns as i32) {
            maxx = fixed_t::new(blockmap.columns as i32);
        }
        if maxy > fixed_t::new(blockmap.rows as i32) {
            maxy = fixed_t::new(blockmap.rows as i32);
        }

        for x in minx.0..maxx.0 {
            for y in miny.0..maxy.0 {
                result.push((x as usize, y as usize));
            }
        }

        result
    }

    fn intercepts(&self, blockmap: &Blockmap, shooter: &MapObject) -> Vec<Intercept> {
        let search_indexes = self.block_march(blockmap);
        let mut intercepts = Vec::new();

        for i in search_indexes {
            let lines = &blockmap.blocklines[(blockmap.columns * i.1) + i.0];
            let mut things = blockmap.thinglist[(blockmap.columns * i.1) + i.0];
            for line in lines {
                if let Some(frac) = self.frac_hit_line(&line.v1, &line.v2) {
                    if frac > FT_ZERO {
                        intercepts.push(Intercept {
                            frac,
                            thing: None,
                            line: Some(line.clone()),
                        })
                    }
                }
            }

            while let Some(thing) = things {
                let the_thing = unsafe { thing.as_ref().unwrap() };
                things = None;
                if let Some(mobj) = the_thing.shootable() {
                    if !core::ptr::eq(mobj, shooter) {
                        let (line1, line2) = mobj.shootable_lines();

                        if check_hit_line(&self.start, &self.end, &line1.0, &line1.1)
                            || check_hit_line(&self.start, &self.end, &line2.0, &line2.1)
                        {
                            let dist_to_hit =
                                ((mobj.xy - self.start).length() - mobj.radius) / self.distance;

                            if dist_to_hit > FT_ZERO {
                                intercepts.push(Intercept {
                                    frac: dist_to_hit,
                                    thing: Some(thing),
                                    line: None,
                                })
                            }
                        }
                    }
                    things = mobj.bm_next;
                }
            }
        }

        intercepts.sort_by(|a, b| a.frac.0.cmp(&b.frac.0));
        intercepts
    }

    pub(crate) fn aim(&self, shooter: &mut MapObject) -> Option<AimHit> {
        let blockmap = &shooter.level().map_data.blockmap;
        let intercepts = self.intercepts(&blockmap, shooter);
        let mut aim_hit = None;

        let mut tl = TraverseLimits {
            max_slope: self.max_aim_slope,
            min_slope: self.min_aim_slope,
        };

        let _ = intercepts
            .into_iter()
            .take_while(|f| {
                if let Some(line) = &f.line {
                    if line.back_sidedef.is_none() {
                        return false;
                    } else {
                        let s1 = &line.frontsector;
                        let s2 = &line.backsector.as_ref().unwrap();
                        let s1c = s1.ceilingheight;
                        let s1f = s1.floorheight;
                        let s2c = s2.ceilingheight;
                        let s2f = s2.floorheight;

                        if s1c == s1f || s2c == s2f {
                            return false;
                        }

                        if s1c <= s2f || s1f >= s2c {
                            return false;
                        }

                        let new_max_slope = if s1c >= s2c {
                            (s2c - self.shoot_z) / (self.distance * f.frac)
                        } else {
                            (s1c - self.shoot_z) / (self.distance * f.frac)
                        };

                        let new_min_slope = if s1f >= s2f {
                            (s1f - self.shoot_z) / (self.distance * f.frac)
                        } else {
                            (s2f - self.shoot_z) / (self.distance * f.frac)
                        };

                        if tl.max_slope > new_max_slope {
                            tl.max_slope = new_max_slope;
                        }
                        if tl.min_slope < new_min_slope {
                            tl.min_slope = new_min_slope;
                        }

                        if tl.min_slope >= tl.max_slope {
                            return false;
                        }
                    }
                } else if let Some(thing) = f.thing {
                    let the_thing = unsafe { thing.as_ref() }.unwrap();
                    if let Some(shootable) = the_thing.shootable() {
                        let top_z = shootable.z + shootable.height;
                        let bot_z = shootable.z;
                        let z_attempt = if top_z >= self.shoot_z && bot_z <= self.shoot_z {
                            self.shoot_z
                        } else {
                            (top_z + bot_z) / FT_TWO
                        };
                        let slope = (z_attempt - self.shoot_z) / (self.distance * f.frac);
                        let hit_dist = self.distance * f.frac;
                        let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                        let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;

                        aim_hit = Some(AimHit {
                            slope,
                            thing,
                            hit_z: z_attempt,
                            hit_location: VecF2::new(hit_loc_x, hit_loc_y),
                        });

                        return false;
                    }
                }
                return true;
            })
            .collect::<Vec<Intercept>>();

        aim_hit
    }

    pub(crate) fn fire<F, G>(
        &self,
        shooter: &mut MapObject,
        level: &mut Level,
        slope: fixed_t,
        damage: i32,
        onhit_wall: &mut F,
        onhit_thing: &mut G,
    ) -> ()
    where
        F: FnMut(fixed_t, fixed_t, fixed_t, fixed_t, &mut Level),
        G: FnMut(
            fixed_t,
            fixed_t,
            fixed_t,
            fixed_t,
            i32,
            Option<&mut MapObject>,
            &mut MapObject,
            &mut Level,
        ),
    {
        let blockmap = &shooter.level().map_data.blockmap;
        let intercepts = self.intercepts(&blockmap, shooter);

        let mut tl = TraverseLimits {
            max_slope: self.max_aim_slope,
            min_slope: self.min_aim_slope,
        };

        let _ = intercepts
            .into_iter()
            .take_while(|f| {
                if let Some(line) = &f.line {
                    if line.back_sidedef.is_none() {
                        let hit_dist = self.distance * f.frac;
                        let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                        let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                        let z_attempt = self.shoot_z + (slope * hit_dist);
                        onhit_wall(hit_loc_x, hit_loc_y, z_attempt, self.distance, level);
                        return false;
                    } else {
                        let s1 = &line.frontsector;
                        let s2 = &line.backsector.as_ref().unwrap();
                        let s1c = s1.ceilingheight;
                        let s1f = s1.floorheight;
                        let s2c = s2.ceilingheight;
                        let s2f = s2.floorheight;

                        if s1c == s1f || s2c == s2f {
                            let hit_dist = self.distance * f.frac;
                            let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                            let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                            let z_attempt = self.shoot_z + (slope * hit_dist);
                            onhit_wall(hit_loc_x, hit_loc_y, z_attempt, self.distance, level);
                            return false;
                        }

                        if s1c <= s2f || s1f >= s2c {
                            let hit_dist = self.distance * f.frac;
                            let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                            let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                            let z_attempt = self.shoot_z + (slope * hit_dist);
                            onhit_wall(hit_loc_x, hit_loc_y, z_attempt, self.distance, level);
                            return false;
                        }

                        let new_max_slope = if s1c >= s2c {
                            (s2c - self.shoot_z) / (self.distance * f.frac)
                        } else {
                            (s1c - self.shoot_z) / (self.distance * f.frac)
                        };

                        let new_min_slope = if s1f >= s2f {
                            (s1f - self.shoot_z) / (self.distance * f.frac)
                        } else {
                            (s2f - self.shoot_z) / (self.distance * f.frac)
                        };

                        if tl.max_slope > new_max_slope {
                            tl.max_slope = new_max_slope;
                        }
                        if tl.min_slope < new_min_slope {
                            tl.min_slope = new_min_slope;
                        }
                        if tl.min_slope >= tl.max_slope {
                            let hit_dist = self.distance * f.frac;
                            let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                            let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                            let z_attempt = self.shoot_z + (slope * hit_dist);
                            onhit_wall(hit_loc_x, hit_loc_y, z_attempt, self.distance, level);
                            return false;
                        }

                        if tl.min_slope > slope || tl.max_slope < slope {
                            let hit_dist = self.distance * f.frac;
                            let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                            let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                            let z_attempt = self.shoot_z + (slope * hit_dist);
                            onhit_wall(hit_loc_x, hit_loc_y, z_attempt, self.distance, level);
                            return false;
                        }
                    }
                } else if let Some(thing) = f.thing {
                    let the_thing = unsafe { thing.as_mut() }.unwrap();
                    if let Some(shootable) = the_thing.shootable() {
                        let top_z = shootable.z + shootable.height;
                        let bot_z = shootable.z;
                        let hit_dist = self.distance * f.frac;
                        let z_attempt = self.shoot_z + (slope * hit_dist);
                        let target_thing = the_thing.mobj_mut();
                        if z_attempt > bot_z && z_attempt < top_z {
                            let hit_loc_x = self.start.x + self.angle.finecos() * hit_dist;
                            let hit_loc_y = self.start.y + self.angle.finesin() * hit_dist;
                            onhit_thing(
                                hit_loc_x,
                                hit_loc_y,
                                z_attempt,
                                self.distance,
                                damage,
                                Some(shooter),
                                target_thing,
                                level,
                            );
                            return false;
                        }
                    }
                }
                return true;
            })
            .collect::<Vec<Intercept>>();
    }

    fn frac_hit_line(&self, line_hit_start: &VecF2, line_hit_end: &VecF2) -> Option<fixed_t> {
        if !check_hit_line(&self.start, &self.end, &line_hit_start, &line_hit_end) {
            return None;
        }
        let dl = Trace::new(*line_hit_start, *line_hit_end - *line_hit_start);
        let trace = Trace::new(self.start, self.end - self.start);
        let i_vector = intercept_vector(trace, dl);
        Some(i_vector)
    }
}

fn check_hit_line(
    hitting_line_start: &VecF2,
    hitting_line_end: &VecF2,
    line_hit_start: &VecF2,
    line_hit_end: &VecF2,
) -> bool {
    let s1 = point_on_side(&hitting_line_start, &hitting_line_end, &line_hit_start);
    let s2 = point_on_side(&hitting_line_start, &hitting_line_end, &line_hit_end);
    s1 != s2
}
