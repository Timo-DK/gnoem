use rand::Rng;
use ratatui::style::Color;

const PREFIXES: &[&str] = &[
    "Grum", "Fiz", "Bram", "Twig", "Nob", "Wort", "Snib", "Burl", "Dink", "Fern", "Gust",
    "Knurl", "Mox", "Pip", "Rook", "Sprig", "Thim", "Wren", "Zib", "Blim",
];

const SUFFIXES: &[&str] = &[
    "bold", "wick", "ble", "knot", "sprout", "whistle", "thorn", "leaf", "bark", "dust", "moss",
    "root", "snap", "spark", "stone", "twitch", "vale", "wort", "crumb", "flint",
];

const COLORS: &[Color] = &[
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightRed,
    Color::LightBlue,
    Color::White,
];

/// Generates a gnomish name by combining a random prefix and suffix.
/// The prefix is already capitalised; the suffix is lowercase.
/// Examples: "Grumbold", "Fizwick", "Twigsnap"
pub fn generate_name(rng: &mut impl Rng) -> String {
    let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
    let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
    format!("{}{}", prefix, suffix)
}

/// Picks a random `ratatui::style::Color` from the available palette.
pub fn generate_color(rng: &mut impl Rng) -> Color {
    COLORS[rng.gen_range(0..COLORS.len())]
}

/// Convenience function that returns both a gnomish name and a colour.
pub fn generate_gnoem_identity(rng: &mut impl Rng) -> (String, Color) {
    (generate_name(rng), generate_color(rng))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn seeded_rng(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    #[test]
    fn generate_name_returns_non_empty_string() {
        let mut rng = seeded_rng(0);
        let name = generate_name(&mut rng);
        assert!(!name.is_empty(), "generated name must not be empty");
    }

    #[test]
    fn generate_name_is_deterministic_with_seeded_rng() {
        let name_a = generate_name(&mut seeded_rng(42));
        let name_b = generate_name(&mut seeded_rng(42));
        assert_eq!(name_a, name_b, "same seed must produce the same name");
    }

    #[test]
    fn generate_name_differs_across_seeds() {
        // With 20 prefixes × 20 suffixes = 400 combinations it is very unlikely
        // that two different seeds produce the exact same name, but we try a few
        // pairs to make the test robust without being flaky.
        let names: Vec<String> = (0u64..10).map(|s| generate_name(&mut seeded_rng(s))).collect();
        let unique: std::collections::HashSet<&String> = names.iter().collect();
        assert!(unique.len() > 1, "different seeds should produce different names");
    }

    #[test]
    fn generate_name_starts_with_known_prefix() {
        let mut rng = seeded_rng(7);
        for _ in 0..50 {
            let name = generate_name(&mut rng);
            let has_known_prefix = PREFIXES.iter().any(|p| name.starts_with(p));
            assert!(has_known_prefix, "name '{}' does not start with a known prefix", name);
        }
    }

    #[test]
    fn generate_name_ends_with_known_suffix() {
        let mut rng = seeded_rng(13);
        for _ in 0..50 {
            let name = generate_name(&mut rng);
            let has_known_suffix = SUFFIXES.iter().any(|s| name.ends_with(s));
            assert!(has_known_suffix, "name '{}' does not end with a known suffix", name);
        }
    }

    #[test]
    fn generate_color_returns_valid_color() {
        let mut rng = seeded_rng(0);
        let color = generate_color(&mut rng);
        assert!(
            COLORS.contains(&color),
            "generated color {:?} is not in the palette",
            color
        );
    }

    #[test]
    fn generate_color_is_deterministic_with_seeded_rng() {
        let color_a = generate_color(&mut seeded_rng(99));
        let color_b = generate_color(&mut seeded_rng(99));
        assert_eq!(color_a, color_b, "same seed must produce the same color");
    }

    #[test]
    fn all_prefixes_are_reachable() {
        // Generate a large number of names and verify every prefix appears at
        // least once. Using a fixed seed keeps the test deterministic while
        // giving a high probability of full coverage (20 prefixes, 2000 draws).
        let mut rng = seeded_rng(1234);
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for _ in 0..2000 {
            let name = generate_name(&mut rng);
            for p in PREFIXES {
                if name.starts_with(p) {
                    seen.insert(p);
                }
            }
        }
        for prefix in PREFIXES {
            assert!(seen.contains(prefix), "prefix '{}' was never generated", prefix);
        }
    }

    #[test]
    fn all_suffixes_are_reachable() {
        let mut rng = seeded_rng(5678);
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for _ in 0..2000 {
            let name = generate_name(&mut rng);
            for s in SUFFIXES {
                if name.ends_with(s) {
                    seen.insert(s);
                }
            }
        }
        for suffix in SUFFIXES {
            assert!(seen.contains(suffix), "suffix '{}' was never generated", suffix);
        }
    }

    #[test]
    fn generate_gnoem_identity_returns_non_empty_name_and_valid_color() {
        let mut rng = seeded_rng(0);
        let (name, color) = generate_gnoem_identity(&mut rng);
        assert!(!name.is_empty(), "identity name must not be empty");
        assert!(COLORS.contains(&color), "identity color {:?} is not in the palette", color);
    }

    #[test]
    fn generate_gnoem_identity_is_deterministic_with_seeded_rng() {
        let (name_a, color_a) = generate_gnoem_identity(&mut seeded_rng(77));
        let (name_b, color_b) = generate_gnoem_identity(&mut seeded_rng(77));
        assert_eq!(name_a, name_b);
        assert_eq!(color_a, color_b);
    }
}
