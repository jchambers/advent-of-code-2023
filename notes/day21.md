```rust
// Hypothesis (from visually inspecting input): there's a "ring" of radial symmetry in here. The start is in the
// center, so the max distance at which we want to look for a ring is the "radius" of the space, or the start
// point's distance from the edge.
let radius = start_index % self.width;

let start_x = start_index % self.width;
let start_y = start_index / self.width;

let mut steps_by_distance_from_start = Vec::from_iter(iter::repeat_with(Vec::new).take(radius + 1));

for (i, steps_to_tile) in distances.iter().enumerate() {
    let x = i % self.width;
    let y = i / self.width;

    let distance = start_x.abs_diff(x) + start_y.abs_diff(y);

    if distance <= radius {
        steps_by_distance_from_start[distance].push(steps_to_tile);
    }
}

steps_by_distance_from_start
    .iter()
    .enumerate()
    .filter(|(_, step_counts)| {
        let first = step_counts.first().unwrap();
        step_counts.iter().all(|c| c == first)
    })
    .for_each(|(i, step_count)| {
        println!("All tiles at distance {} reachable in {} steps", i, step_count.first().unwrap());
    });
```

…and it turns out there IS radial symmetry at 64 and 65! The latter is probably more interesting, since that takes us to the very edge of the repeating pattern.

It also turns out that there's an empty border around the pattern (both in the real and example data), so the edges of each pattern have a predictable pattern (65 in the center, ramping up smoothly to 130 in the corners).

So what happens as we expand outward in a radially-symmetric world?

```
.........
.BB...BB.
.B.....B.
....A....
...AAA...
....A....
.B.....B.
.BB...BB.
.........
```

…turns into:

```
...........................
.BB...BB..BB...BB..BB...BB.
.B.....B..B.....B..B.....B.
....A........A........A....
...AAA......AAA......AAA...
....A........A........A....
.B.....B..B.....B..B.....B.
.BB...BB..BB...BB..BB...BB.
...........................
...........................
.BB...BB..BB...BB..BB...BB.
.B.....B..B.....B..B.....B.
....A........A........A....
...AAA......AAA......AAA...
....A........A........A....
.B.....B..B.....B..B.....B.
.BB...BB..BB...BB..BB...BB.
...........................
...........................
.BB...BB..BB...BB..BB...BB.
.B.....B..B.....B..B.....B.
....A........A........A....
...AAA......AAA......AAA...
....A........A........A....
.B.....B..B.....B..B.....B.
.BB...BB..BB...BB..BB...BB.
...........................
```

Taking the two observations about the input data's special properties together, we can:

1. Figure out the "radius" of the "world" in repeating zones: $\frac{26501365 - 65}{131} = 202300$
2. Find the distance to an appropriate starting point in each appropriate zone (i.e. the center of the right edge for zones directly to the left of the starting tile, or the bottom left corner of any zone above and to the right of the starting zone)
3. For each of the 9 prototype zones (the 8 starting positions for repeated zones plus the starting zone), figure out how often they repeat
4. Deal with "chopped" zones on the edges (though those should also repeat)
5. Multiply! Add! Math!

Looks like we need to generalize a bit for the test data (which honestly makes me feel better). The test data does NOT have radial symmetry, but it does have the empty border. That SHOULD mean that we can do the "nine repeating patterns" thing, just with slightly less predictable starting points. Or maybe we need to generalize even further and cache edge values…?

One interesting outcome from the "everything has an obstacle-free border" observation is that an explorer starting in one corner has a guaranteed, optimal path to ~the opposite~ any other corner by traveling along the edges. I'm pretty sure this also guarantees that we can always consider each repeated zone independently (i.e. there's never going to be a secret shortcut in from some unexpected tile).

Also interesting: going from corner to corner, we'll never do an even/odd "flip-flop," so we don't have to worry about alternating patterns of reachability. As long as we start in the same corner, the same tiles will be reachable.

For each quadrant, we'll have a triangle (possibly with zero size) of repeating zones. That zone will have an "inner core" of fully-reachable zones with an outer "crust" of potentially partially-reachable zones. The partially-reachable zones within a quadrant will all have the same reachability because the distance offset at the starting corner will be the same in all cases.

From there, we have four "spokes" to contend with. In the example data, they may or may not stabilize into a repeating pattern. In the real problem data, they stabilize into a repeating pattern immediately.

```
    |
   #|#
  ##|##
 ###|###
----+----
 ###|###
  ##|##
   #|#
    |
```
