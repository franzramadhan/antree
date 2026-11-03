//! Obstacle rolled on arrival at a counter, GDD section 6.

use serde::Deserialize;

use super::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Obstacle {
    Smooth,
    SystemDown,
    ClerkBreak,
    MissingPaperwork,
    FormChanged,
    FixerOffer,
    CounterMoved,
    CollectiveLeave,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mods {
    /// The player declined a bribe at this office earlier in the run.
    pub declined: bool,
    /// The player called the inside man at some point in the run.
    pub connected: bool,
    /// The player paid grease money at this office today.
    pub greased: bool,
}

/// Weighted roll. A weight of 0 stays 0 under every modifier, so an office can
/// rule an obstacle out entirely.
pub fn roll(rng: &mut Rng, weights: &[(Obstacle, u32)], mods: Mods) -> Obstacle {
    if mods.greased {
        return Obstacle::Smooth;
    }
    let adjusted: Vec<(Obstacle, u32)> = weights
        .iter()
        .map(|&(o, w)| {
            if o == Obstacle::Smooth || w == 0 {
                return (o, w);
            }
            let w = if mods.declined { w + 20 } else { w };
            (o, if mods.connected { (w / 2).max(1) } else { w })
        })
        .collect();
    let total: u32 = adjusted.iter().map(|&(_, w)| w).sum();
    let mut r = rng.range(0, total);
    for (o, w) in adjusted {
        if r < w {
            return o;
        }
        r -= w;
    }
    unreachable!("r < total")
}

#[cfg(test)]
mod tests {
    use super::*;
    use Obstacle::*;

    const DEFAULT: [(Obstacle, u32); 8] = [
        (Smooth, 25),
        (SystemDown, 15),
        (ClerkBreak, 15),
        (MissingPaperwork, 15),
        (FormChanged, 10),
        (FixerOffer, 10),
        (CounterMoved, 5),
        (CollectiveLeave, 5),
    ];

    fn share(o: Obstacle, mods: Mods) -> f64 {
        let mut rng = Rng::new(11);
        let n = 10_000;
        let hits = (0..n)
            .filter(|_| roll(&mut rng, &DEFAULT, mods) == o)
            .count();
        hits as f64 / n as f64
    }

    #[test]
    fn distribution_matches_weights_within_two_percent() {
        for (o, w) in DEFAULT {
            let got = share(o, Mods::default());
            assert!((got - w as f64 / 100.0).abs() < 0.02, "{o:?}: {got}");
        }
    }

    #[test]
    fn grease_forces_smooth() {
        let mods = Mods {
            greased: true,
            ..Mods::default()
        };
        assert_eq!(share(Smooth, mods), 1.0);
    }

    #[test]
    fn declining_makes_trouble_likelier_and_contacts_make_it_rarer() {
        let base = share(Smooth, Mods::default());
        let declined = Mods {
            declined: true,
            ..Mods::default()
        };
        let connected = Mods {
            connected: true,
            ..Mods::default()
        };
        assert!(share(Smooth, declined) < base - 0.05);
        assert!(share(Smooth, connected) > base + 0.05);
    }

    #[test]
    fn zero_weight_never_rolls() {
        let w = [(Smooth, 1), (CollectiveLeave, 0)];
        let mods = Mods {
            declined: true,
            ..Mods::default()
        };
        let mut rng = Rng::new(1);
        assert!((0..1000).all(|_| roll(&mut rng, &w, mods) == Smooth));
    }
}
