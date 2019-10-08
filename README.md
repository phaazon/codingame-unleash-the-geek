# Unleash the Geek

## v0.1

For this first version, I wanted:

- To discover the game mechanics around mining.
- Have a single unit handling radars. That miner is not necessarily the same one every time.
  instead, I pick the “best” one when I need a radar (i.e. it’s the one nearest a HQ cell) then
  dispatch it to grab a radar and deploy it at a random location. I don’t take cooldown into account
  so far.
- All other units are dispatched according to a simple rule:
  - If we have ore information, we know for sure that some ore is availble, so we take the nearest
    ore cell to the miner and dispatch the miner to dig there.
  - If no ore information is available (typical when starting the game), we go in a complete random
    location.
- When a unit either burries a radar or digs, if it has found ore, it immediately goes back to HQ.
- We don’t use traps at all so far nor we defend against enemies nor attack them. Our current
  strategy is mining first and multiplying the number of radars we dig. No defence / attack
  strategy for v1.0.

Possible enhancements:

- When a radar-miner is dispatched, it picks a random location to deploy the radar. It’s a bit dumb
  as we might want to have a better idea of the map. Instead, we need to find the “best place” in
  on the grid we already know to cover as much area as possible. Also, we should explore first near
  HQ and then try to go far away.
- When choosing a new order for a miner that has finished their job, we need to be smarter when
  picking a new place to dig. Currently, we go to the “nearest ore cell” to the unit, which is
  completely fine BUT we need to ensure that given the amount of ore we know this ore cell has,
  going there won’t overcrowd the cell. Implementing this idea will bring a nice heuristics and a
  smoother repartition on the map.
- We can try to burry a radar and unburry it immediately. This will have the effect of allowing to
  explore way quicker but we lose “live” information, so we need to change a bit the way the cells
  store information.

## v0.2

- Radars are now deployed at 3 cells far from edges.
