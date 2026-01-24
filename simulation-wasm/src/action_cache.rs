//! Action Template Resolution Cache
//!
//! This module provides caching for resolved action templates (FinalActions).
//! Templates are resolved once and cached based on template name and overrides,
//! allowing reuse across simulation runs unless creature/override changes.

use crate::model::action::{ResolvedAction, TemplateAction};
use crate::model::DiceFormula;
use crate::enums::{AllyTarget, EnemyTarget, TargetType};
use std::cell::RefCell;
use std::collections::HashMap;

/// Cache key for template resolution.
/// Includes template name and all override values that affect resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateCacheKey {
    pub template_name: String,
    pub overrides: Vec<crate::model::action::TemplateOverride>,
}

impl TemplateCacheKey {
    pub fn new(template_name: &str, options: &crate::model::action::TemplateOptions) -> Self {
        let mut sorted_overrides = options.overrides.clone();
        sorted_overrides.sort_by_key(|o| match o {
            crate::model::action::TemplateOverride::SaveDC(_) => 0,
            crate::model::action::TemplateOverride::Amount(_) => 1,
            crate::model::action::TemplateOverride::Target(_) => 2,
        });

        Self {
            template_name: template_name.to_lowercase(),
            overrides: sorted_overrides,
        }
    }
}

// Global cache for resolved action templates
thread_local! {
    static TEMPLATE_CACHE: RefCell<HashMap<TemplateCacheKey, ResolvedAction>> = RefCell::new(HashMap::new());
}

/// Template resolver that converts TemplateActions to ResolvedActions
pub struct ActionTemplateResolver;

impl ActionTemplateResolver {
    /// Resolve a template action to a concrete resolved action.
    /// This implements the template resolution logic based on template name.
    pub fn resolve_template(template_action: &TemplateAction) -> Result<ResolvedAction, String> {
        let template_name = template_action.template_options.template_name.to_lowercase();
        let key = TemplateCacheKey::new(&template_name, &template_action.template_options);

        // Check cache first
        if let Some(cached) = Self::get_cached(&key) {
            return Ok(cached.clone());
        }

        // Extract overrides for easier access
        let overrides = &template_action.template_options.overrides;

        // Resolve the template
        let resolved = match template_name.as_str() {
            "bless" => Self::resolve_bless(template_action, overrides),
            "bane" => Self::resolve_bane(template_action, overrides),
            "haste" => Self::resolve_haste(template_action, overrides),
            "shield" => Self::resolve_shield(template_action, overrides),
            "hunter's mark" => Self::resolve_hunters_mark(template_action, overrides),
            "hex" => Self::resolve_hex(template_action, overrides),
            "hypnotic pattern" => Self::resolve_hypnotic_pattern(template_action, overrides),
            _ => return Err(format!("Unknown template: {}", template_name)),
        }?;

        // Cache the result
        Self::cache_result(key, resolved.clone());

        Ok(resolved)
    }

    /// Get a cached resolved action
    fn get_cached(key: &TemplateCacheKey) -> Option<ResolvedAction> {
        TEMPLATE_CACHE.with(|cache| cache.borrow().get(key).cloned())
    }

    /// Cache a resolved action
    fn cache_result(key: TemplateCacheKey, action: ResolvedAction) {
        TEMPLATE_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            // Limit cache size to prevent memory issues
            const MAX_CACHE_ENTRIES: usize = 1000;
            if cache.len() >= MAX_CACHE_ENTRIES {
                cache.clear();
            }
            cache.insert(key, action);
        });
    }

    /// Clear the template cache
    pub fn clear_cache() {
        TEMPLATE_CACHE.with(|cache| cache.borrow_mut().clear());
    }

    /// Get cache statistics
    pub fn get_cache_stats() -> (usize, usize) {
        TEMPLATE_CACHE.with(|cache| {
            let cache = cache.borrow();
            let entry_count = cache.len();
            // Estimate ~500 bytes per entry (rough average)
            let estimated_bytes = entry_count * 500;
            (entry_count, estimated_bytes)
        })
    }

    // Template resolution implementations

    fn resolve_bless(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_buff_target_from_overrides(overrides, AllyTarget::AllyWithLeastHP);

        Ok(ResolvedAction::Buff(crate::model::BuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            buff: crate::model::Buff {
                display_name: Some("Bless".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: Some(DiceFormula::Expr("1d4".to_string())),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(DiceFormula::Expr("1d4".to_string())),
                condition: None,
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(),
                suppressed_until: None,
            },
        }))
    }

    fn resolve_bane(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_debuff_target_from_overrides(overrides, EnemyTarget::EnemyWithLeastHP);
        let save_dc = Self::extract_save_dc(overrides).unwrap_or(13.0);

        Ok(ResolvedAction::Debuff(crate::model::DebuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            save_dc,
            buff: crate::model::Buff {
                display_name: Some("Bane".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: Some(DiceFormula::Expr("-1d4".to_string())),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(DiceFormula::Expr("-1d4".to_string())),
                condition: None,
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(),
                suppressed_until: None,
            },
        }))
    }

    fn resolve_haste(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_buff_target_from_overrides(overrides, AllyTarget::AllyWithLeastHP);

        Ok(ResolvedAction::Buff(crate::model::BuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            buff: crate::model::Buff {
                display_name: Some("Haste".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: Some(DiceFormula::Value(2.0)),
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(),
                suppressed_until: None,
            },
        }))
    }

    fn resolve_shield(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_buff_target_from_overrides(overrides, AllyTarget::Self_);

        Ok(ResolvedAction::Buff(crate::model::BuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            buff: crate::model::Buff {
                display_name: Some("Shield".to_string()),
                duration: crate::enums::BuffDuration::OneRound,
                ac: Some(DiceFormula::Value(5.0)),
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None,
                concentration: false,
                triggers: Vec::new(),
                suppressed_until: None,
            },
        }))
    }

    fn resolve_hunters_mark(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_debuff_target_from_overrides(overrides, EnemyTarget::EnemyWithLeastHP);

        Ok(ResolvedAction::Debuff(crate::model::DebuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            save_dc: Self::extract_save_dc(overrides).unwrap_or(13.0),
            buff: crate::model::Buff {
                display_name: Some("Hunter's Mark".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(), // Would need to add damage bonus triggers
                suppressed_until: None,
            },
        }))
    }

    fn resolve_hex(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_debuff_target_from_overrides(overrides, EnemyTarget::EnemyWithLeastHP);

        Ok(ResolvedAction::Debuff(crate::model::DebuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            save_dc: Self::extract_save_dc(overrides).unwrap_or(13.0),
            buff: crate::model::Buff {
                display_name: Some("Hex".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(), // Would need to add damage bonus triggers
                suppressed_until: None,
            },
        }))
    }

    fn resolve_hypnotic_pattern(template_action: &TemplateAction, overrides: &[crate::model::action::TemplateOverride]) -> Result<ResolvedAction, String> {
        let target = Self::resolve_debuff_target_from_overrides(overrides, EnemyTarget::EnemyWithLeastHP);

        Ok(ResolvedAction::Debuff(crate::model::DebuffAction {
            id: template_action.id.clone(),
            name: template_action.name.clone(),
            action_slot: template_action.action_slot,
            cost: template_action.cost.clone(),
            requirements: template_action.requirements.clone(),
            tags: template_action.tags.clone(),
            freq: template_action.freq.clone(),
            condition: template_action.condition.clone(),
            targets: template_action.targets,
            target,
            save_dc: Self::extract_save_dc(overrides).unwrap_or(13.0),
            buff: crate::model::Buff {
                display_name: Some("Hypnotic Pattern".to_string()),
                duration: crate::enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: Some(crate::enums::CreatureCondition::Incapacitated),
                magnitude: None,
                source: None,
                concentration: true,
                triggers: Vec::new(),
                suppressed_until: None,
            },
        }))
    }

    /// Extract save DC from overrides
    fn extract_save_dc(overrides: &[crate::model::action::TemplateOverride]) -> Option<f64> {
        overrides.iter().find_map(|o| match o {
            crate::model::action::TemplateOverride::SaveDC(dc) => Some(*dc as f64),
            _ => None,
        })
    }

    /// Extract amount from overrides
    fn extract_amount(overrides: &[crate::model::action::TemplateOverride]) -> Option<&DiceFormula> {
        overrides.iter().find_map(|o| match o {
            crate::model::action::TemplateOverride::Amount(formula) => Some(formula),
            _ => None,
        })
    }

    /// Extract target from overrides
    fn extract_target(overrides: &[crate::model::action::TemplateOverride]) -> Option<&TargetType> {
        overrides.iter().find_map(|o| match o {
            crate::model::action::TemplateOverride::Target(target) => Some(target),
            _ => None,
        })
    }

    /// Resolve the target type for buff actions
    fn resolve_buff_target_from_overrides(overrides: &[crate::model::action::TemplateOverride], default_target: AllyTarget) -> AllyTarget {
        match Self::extract_target(overrides) {
            Some(TargetType::Ally(ally_target)) => ally_target.clone(),
            _ => default_target,
        }
    }

    /// Resolve the target type for debuff actions
    fn resolve_debuff_target_from_overrides(overrides: &[crate::model::action::TemplateOverride], default_target: EnemyTarget) -> EnemyTarget {
        match Self::extract_target(overrides) {
            Some(TargetType::Enemy(enemy_target)) => enemy_target.clone(),
            _ => default_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::action::{TemplateAction, TemplateOptions, TemplateOverride};
    use crate::model::{Frequency, ActionCondition, DiceFormula};
    use crate::enums::TargetType;

    #[test]
    fn test_template_cache_key_creation() {
        let options = TemplateOptions {
            template_name: "bless".to_string(),
            target: None,
            save_dc: None,
            amount: None,
            overrides: vec![
                TemplateOverride::Target(TargetType::Ally(AllyTarget::AllyWithLeastHP)),
                TemplateOverride::SaveDC(15.0),
                TemplateOverride::Amount(DiceFormula::Value(10.0)),
            ],
        };

        let key = TemplateCacheKey::new("bless", &options);
        assert_eq!(key.template_name, "bless");
        assert_eq!(key.overrides.len(), 3);
    }

    #[test]
    fn test_resolve_bless_template() {
        let template = TemplateAction {
            id: "test_bless".to_string(),
            name: "Bless".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("1/day".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            template_options: TemplateOptions {
                template_name: "bless".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![
                    TemplateOverride::Target(TargetType::Ally(AllyTarget::AllyWithLeastHP)),
                ],
            },
        };

        let result = ActionTemplateResolver::resolve_template(&template);
        assert!(result.is_ok());

        if let Ok(ResolvedAction::Buff(buff_action)) = result {
            assert_eq!(buff_action.id, "test_bless");
            assert_eq!(buff_action.name, "Bless");
            assert_eq!(buff_action.buff.display_name, Some("Bless".to_string()));
        } else {
            panic!("Expected Buff action");
        }
    }

    #[test]
    fn test_resolve_unknown_template() {
        let template = TemplateAction {
            id: "test_unknown".to_string(),
            name: "Unknown".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            template_options: TemplateOptions {
                template_name: "unknown_spell".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![],
            },
        };

        let result = ActionTemplateResolver::resolve_template(&template);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown template"));
    }

    #[test]
    fn test_template_caching() {
        // Clear cache first
        ActionTemplateResolver::clear_cache();

        let template = TemplateAction {
            id: "test_shield".to_string(),
            name: "Shield".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            template_options: TemplateOptions {
                template_name: "shield".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![],
            },
        };

        // First resolution should cache
        let result1 = ActionTemplateResolver::resolve_template(&template);
        assert!(result1.is_ok());

        // Check cache has entry
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 1);

        // Second resolution should use cache
        let result2 = ActionTemplateResolver::resolve_template(&template);
        assert!(result2.is_ok());
        assert_eq!(result1, result2);

        // Clear cache
        ActionTemplateResolver::clear_cache();
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_cache_invalidation_on_override_changes() {
        // Clear cache first
        ActionTemplateResolver::clear_cache();

        // Template with no overrides
        let template_base = TemplateAction {
            id: "test_bane".to_string(),
            name: "Bane".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            template_options: TemplateOptions {
                template_name: "bane".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![],
            },
        };

        // Template with save DC override
        let template_with_dc = TemplateAction {
            id: "test_bane".to_string(),
            name: "Bane".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            template_options: TemplateOptions {
                template_name: "bane".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![
                    TemplateOverride::SaveDC(15.0),
                ],
            },
        };

        // Template with different save DC
        let template_with_different_dc = TemplateAction {
            id: "test_bane".to_string(),
            name: "Bane".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            template_options: TemplateOptions {
                template_name: "bane".to_string(),
                target: None,
                save_dc: None,
                amount: None,
                overrides: vec![
                    TemplateOverride::SaveDC(16.0),
                ],
            },
        };

        // Resolve base template - should cache
        let result1 = ActionTemplateResolver::resolve_template(&template_base);
        assert!(result1.is_ok());
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 1);

        // Resolve template with DC override - should create new cache entry
        let result2 = ActionTemplateResolver::resolve_template(&template_with_dc);
        assert!(result2.is_ok());
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 2);

        // Resolve template with different DC - should create another cache entry
        let result3 = ActionTemplateResolver::resolve_template(&template_with_different_dc);
        assert!(result3.is_ok());
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 3);

        // Resolve base template again - should use existing cache
        let result4 = ActionTemplateResolver::resolve_template(&template_base);
        assert!(result4.is_ok());
        assert_eq!(result1, result4);

        // Results with different overrides should be different (different save DCs)
        if let (Ok(ResolvedAction::Debuff(debuff1)), Ok(ResolvedAction::Debuff(debuff2))) = (&result1, &result2) {
            assert_ne!(debuff1.save_dc, debuff2.save_dc);
        }
        if let (Ok(ResolvedAction::Debuff(debuff2)), Ok(ResolvedAction::Debuff(debuff3))) = (&result2, &result3) {
            assert_ne!(debuff2.save_dc, debuff3.save_dc);
        }

        // Clear cache
        ActionTemplateResolver::clear_cache();
        let (count, _) = ActionTemplateResolver::get_cache_stats();
        assert_eq!(count, 0);
    }
}