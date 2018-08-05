/*
    Computes the [0,1] multiplier for a given level. This is a logarithmic curve mixed with a
    linear curve, such that advancement is fastest at the beginning, going from 0.0->0.28 in the
    first two levels, from 0.28->0.5 by level 5, 0.5->0.75 by level 10, 0.75 -> 1.0 by level 20.
    (log[4.5](x+1) * 0.5 * 0.7) + ((x+1)/20) * 0.3 - 0.015
    the 0.7/0.3 is the weighting between log and linear, the sub 0.015 shifts down to 0 at lvl 0
*/
pub fn level_curve(level : u32) -> f64 {
    let x = level as f64 + 1.0;
    f64::log(x, 4.5) * 0.5 * 0.7 + (x/20.0) * 0.3 - 0.015
}


// thoughts:

// one question is what do we want the range to be on skills, [0,20]? [0,9]?
// I think [0,20], but 20 is godlike and virtually unattainable, that would be someone naturally gifted
// with an appropriate background, who you keep with you, focused on the relevant job for the entire
// campaign. So [0,9] is the common range. In keeping with our focus gives increasing benefits idea though
// it ought to give ever increasing benefit in order to make it tempting... but we still assume that it
// takes 2x as much xp to get each subsequent level? Or d20 Style
// levels  1  2  3  4   5   6   7   8    9   10  11  12  13  14
// that's: 1, 2, 4, 8,  16, 32, 64, 128, 256
// vs    : 1, 3, 6, 10, 15, 21, 28, 36,  45, 55, 66, 78, 91, 105
// (note, (0.5 * level^2 - 0.5 * level) gives you the above advancement)
// I think that the d20 style leveling advancement makes more sense, it continues to get harder
// but it doesn't have that same exponential curve to infinity. Ok, so we've got a < x^2 difficulty
// to advance each level, but realistically I don't think we can go much more than linear improvement
// in outcomes or it becomes ridiculous, if you got an x^2 damage bonus it would get bonkers. One
// thing I would like to avoid, I think, is utterly rescaling later levels relative to early ones.
// The situations I find unsatisfying there are 1) when numbers get stupid large they become difficult
// to reason about usefully and feel unsatisfying. When you're dealing 1244423 damage, it just becomes
// noise, 2) when enemies scale up to meet you without changing substance it feels like you're on a
// treadmill and advancement is useless, if you get to the last level and there's a lvl 20 rabbit
// with 10x as much hp as a rabbit in the first level, it seems like, why bother leveling?
// So I can't just scale up enemies arbitrarily, we want the dynamic range to be important, but not
// insane. Currently we're looking at a logistic curve, or s curve or what have you, starts of
// exponentially improving up through about 7 at which point it's linear up through about 12 where
// it becomes more logarithmic. The raw curve is a bit too extreme, so we're going to mix it with
// a straight linear, such that we still get some progress up in the 17-20 range, though diminished
// and the advancement in the 1-4 range isn't so pronounced...actually the s curve is _not_ what
// we want here, that has slowest advancement at beginning and end, fastest in the middle. We
// probably do want a plain old logarithmic advancement.

/*
    Now here's a crazy one, not saying we should actually do it, but just a thought exercise. What
    if we didn't increase any of the scales or multipliers directly with level, but rather gave
    specific perks per-level. I.e. gaining a level of melee grants you one of n perks, like,
    +1 damage, or +1 to-hit, or +2 to-hit against enemies in this condition, etc. It's similar
    to the approach that that custom wesnoth campaign uses, I think. The goal though is to make
    each of the levels in some way meaningful, rather than just 15% better or whatever. The other
    way to go about that is through the class leveling system, so skills level up quickly-ish
    and grant continuously increasing gradient bonuses, but class levels are gained more infrequently
    and the class choices are determined by skills and offer more meaningful choices. If you level
    up ranged attack and forest survival you gain access to the ranger class, which grants more
    interesting perks.

    Or both, I suppose, that's the other possibility. But if we were going to go that route it would
    just be a lot of overhead, choosing a perk at every skill up, and choosing classes at every
    class level up, and so on. We could scale between by doing a continuous gradient improvement,
    with perk choices at intervals, i.e. odd levels, or 5,10,15 or whatever. So you get continuously
    a little better at everything, but you get to make distinct choices periodically. You could go
    into a weird meta-loop by also making perk choices dependent on other skills or classes, so it
    all interlinks together. You could also have the perks be hardcoded and not up for choice, so
    you get continuously a little better, then gain noteworthy perks at intervals, without the
    need for concrete choice. Leveling melee attack to 5 grants the ability to do a power attack,
    ranged to 3 gets you aimed shot, or dodge 5 gives you a passive chance to reflex save away from
    area of effect attacks. I feel like that's probably the best approach, otherwise the choices
    become overwhelming. You're choosing which skills to level (by usage or training), which directly
    determines which perks you get (since they're fixed), and determines the options when class
    leveling occurs, which then is a secondary choice.
*/