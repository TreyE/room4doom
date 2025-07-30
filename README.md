# ROOM - Rusty Doom

An attempt at a limit-removing compatible Doom in something other than C.

Based on, but deviates significantly from, work by the awesome [flukejones](https://github.com/flukejones/room4doom).

His original project readme and details are [here](./flukejones_AWESOME_README.md).

## Changed over to Fixed Point Math and Reinstated Blockmap

I've changed the original project over to use fixed point math, like the original DOOM, as well as use the blockmap again.

Since then, the following is broken:
1. Shoot Line of Sight for select monsters is broken
2. The "OOF" crouch when falling from great heights
3. Monsters turn instantly instead of partially per tick
4. Collision checking doesn't correctly allow line-skips and item grab/bumps
5. Sound no longer floods correctly between sectors
6. Aspect ratios are currently fixed, I'm not supporting widescreen.
7. Aim Slope limiting for the player is incorrect (it's to big)
8. Slope checking for aiming/hitting of things isn't exactly correct - it doesn't yet take into account the top/bottom of the thing vs the allowed slopes due to the placement of walls
9. I have no idea the status of demos
