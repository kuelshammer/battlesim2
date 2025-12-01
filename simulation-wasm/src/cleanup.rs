use crate::model::*;
use std::collections::HashSet;

pub fn remove_dead_buffs(targets: &mut [Combattant], dead_source_ids: &HashSet<String>) {
    if dead_source_ids.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("CLEANUP: No dead sources to process.");
        return;
    }

    #[cfg(debug_assertions)]
    eprintln!("CLEANUP: Removing buffs from dead sources: {:?}", dead_source_ids);

    for target in targets.iter_mut() {
        let _before_count = target.final_state.buffs.len();
        target.final_state.buffs.retain(|_buff_id, buff| {
            if let Some(source) = &buff.source {
                let should_keep = !dead_source_ids.contains(source);
                if !should_keep {
                    #[cfg(debug_assertions)]
                    eprintln!("CLEANUP: Removing buff '{}' from {} (source {} is dead)",
                        _buff_id, target.creature.name, source);
                }
                should_keep
            } else {
                // Buff with no source is always kept (might be innate effects)
                true
            }
        });
        let _after_count = target.final_state.buffs.len();

        #[cfg(debug_assertions)]
        if _before_count != _after_count {
            eprintln!("CLEANUP: {} had {} buffs, now has {} buffs",
                target.creature.name, _before_count, _after_count);
        }
    }
}
