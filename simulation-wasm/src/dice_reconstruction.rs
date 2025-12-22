pub fn reconstruct_hp(target_hp: f64, hit_die_size: u32, con_mod: i32) -> String {
    let avg_die = (hit_die_size as f64 + 1.0) / 2.0;
    let die_count = (target_hp / (avg_die + con_mod as f64)).round() as u32;
    let die_count = die_count.max(1);
    
    let base = die_count as i32 * con_mod;
    if base == 0 {
        format!("{}d{}", die_count, hit_die_size)
    } else if base > 0 {
        format!("{}d{} + {}", die_count, hit_die_size, base)
    } else {
        format!("{}d{} - {}", die_count, hit_die_size, base.abs())
    }
}

pub fn reconstruct_damage(target_dpr: f64, ability_mod: i32) -> String {
    let target_dice_damage = (target_dpr - ability_mod as f64).max(1.0);
    
    // Choose best die fit
    let die_options = [(3.5, "d6"), (4.5, "d8"), (5.5, "d10"), (6.5, "d12"), (2.5, "d4")];
    
    let mut best_fit_name = "d6";
    let mut best_fit_avg = 3.5;
    let mut min_diff = f64::MAX;
    
    for (avg, name) in die_options {
        let count = (target_dice_damage / avg).round() as u32;
        let count = count.max(1);
        let diff = (target_dice_damage - (count as f64 * avg)).abs();
        if diff < min_diff {
            min_diff = diff;
            best_fit_name = name;
            best_fit_avg = avg;
            if diff < 0.1 { break; }
        }
    }
    
    let count = (target_dice_damage / best_fit_avg).round() as u32;
    let count = count.max(1);

    if ability_mod == 0 {
        format!("{}{} ", count, best_fit_name)
    } else if ability_mod > 0 {
        format!("{}{} + {}", count, best_fit_name, ability_mod)
    } else {
        format!("{}{} - {}", count, best_fit_name, ability_mod.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconstruct_hp() {
        // Ogre: 59 HP, 7d10 + 21
        let s = reconstruct_hp(59.0, 10, 3);
        assert_eq!(s, "7d10 + 21");
    }

    #[test]
    fn test_reconstruct_damage() {
        // Ogre Greatclub: 13 dmg, 2d8 + 4
        let s = reconstruct_damage(13.0, 4);
        assert_eq!(s, "2d8 + 4");
    }
}
