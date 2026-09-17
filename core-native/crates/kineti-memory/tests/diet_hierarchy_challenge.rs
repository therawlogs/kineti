use kineti_memory::{
    resolve_advice, ActionEvaluation, ConsequenceLevel, ContextScope, DomainKind,
    EpistemicCertainty, EpistemicFact, RuleConstraintType,
};

fn make_fact(
    id: &str,
    claim: &str,
    constraint_type: RuleConstraintType,
    attribute: &str,
) -> EpistemicFact {
    EpistemicFact {
        id: id.to_string(),
        user_id: "test_user".to_string(),
        scope: ContextScope::Domain(DomainKind::Health),
        attribute: attribute.to_string(),
        claim: claim.to_string(),
        constraint_type,
        certainty: EpistemicCertainty::DirectlyKnown,
        valid_from: 100,
        valid_until: None,
        contradiction_criteria: None,
        consequence_level: match constraint_type {
            RuleConstraintType::Preference => ConsequenceLevel::Operational,
            _ => ConsequenceLevel::HighConsequence,
        },
    }
}

/// Baseline scenario: "Vegetarian + Eats Eggs + Low Dairy + Peanut Allergy"
#[test]
fn test_complex_permutations_vegetarian_eggs_dairy_peanut() {
    let facts = vec![
        make_fact("f1", "Vegetarian", RuleConstraintType::BaselineRule, "diet"),
        make_fact("f2", "eggs", RuleConstraintType::PermittedException, "diet_exc"),
        make_fact("f3", "low dairy", RuleConstraintType::Preference, "dairy_pref"),
        make_fact("f4", "peanut", RuleConstraintType::SafetyCeiling, "allergy"),
    ];

    // 1a. Pure Meat (Chicken) -> MUST be blocked by Vegetarian rule
    let chicken = resolve_advice(&facts, "Grilled Chicken", &["meat", "chicken"]);
    assert!(
        matches!(chicken.evaluation, ActionEvaluation::BlockedByRule { ref rule } if rule.contains("vegetarian")),
        "Expected chicken to be blocked by vegetarian rule, got {:?}",
        chicken.evaluation
    );

    // 1b. Pure Eggs -> MUST be permitted via exception
    let eggs = resolve_advice(&facts, "Scrambled Eggs", &["eggs"]);
    assert!(
        matches!(eggs.evaluation, ActionEvaluation::Permitted { .. }),
        "Expected eggs to be permitted via exception, got {:?}",
        eggs.evaluation
    );

    // 1c. Pure Peanut -> MUST be blocked by safety ceiling
    let peanut = resolve_advice(&facts, "Peanut Butter", &["peanut"]);
    assert!(
        matches!(peanut.evaluation, ActionEvaluation::BlockedBySafetyCeiling { ref reason } if reason.contains("peanut")),
        "Expected peanut to be blocked by safety ceiling, got {:?}",
        peanut.evaluation
    );

    // 1d. Dairy item (Cheese Tortellini) -> MUST be permitted with preference, NOT treated as allergy
    let cheese = resolve_advice(&facts, "Cheese Tortellini", &["vegetarian", "dairy", "cheese"]);
    match cheese.evaluation {
        ActionEvaluation::Permitted { ref recommendations } => {
            assert!(
                recommendations.iter().any(|r| r.contains("low dairy")),
                "Expected low dairy recommendation, got {:?}",
                recommendations
            );
        }
        other => panic!("Expected Cheese to be Permitted with preference, but got {:?}", other),
    }

    // 1e. Eggs + Peanut (e.g. egg dish garnished with peanuts) -> SafetyCeiling MUST take precedence over exception
    let egg_peanut = resolve_advice(&facts, "Egg Noodle with Peanut Satay", &["eggs", "peanut"]);
    assert!(
        matches!(egg_peanut.evaluation, ActionEvaluation::BlockedBySafetyCeiling { .. }),
        "SafetyCeiling MUST take precedence over PermittedException when both are present! Got: {:?}",
        egg_peanut.evaluation
    );

    // 1f. Eggs + Chicken (tags: ["chicken", "eggs"]) -> Chicken is blocked even though eggs are permitted
    let chicken_eggs_1 = resolve_advice(&facts, "Chicken & Egg Rice Bowl", &["chicken", "eggs"]);
    assert!(
        matches!(chicken_eggs_1.evaluation, ActionEvaluation::BlockedByRule { .. }),
        "Chicken must block even though eggs are permitted. Got: {:?}",
        chicken_eggs_1.evaluation
    );

    // 1g. Eggs + Chicken (tags: ["eggs", "chicken"] - reversed tag order!)
    let chicken_eggs_2 = resolve_advice(&facts, "Egg & Chicken Rice Bowl", &["eggs", "chicken"]);
    assert!(
        matches!(chicken_eggs_2.evaluation, ActionEvaluation::BlockedByRule { .. }),
        "Tag order ['eggs', 'chicken'] must NOT bypass vegetarian rule for chicken! Got: {:?}",
        chicken_eggs_2.evaluation
    );
}

/// Adversarial Test: Ingredient matches BOTH an exception AND a safety ceiling.
/// SafetyCeiling MUST take precedence over exceptions!
/// Also checks whether prompt directives contain conflicting text snippets.
#[test]
fn test_same_ingredient_exception_and_safety_ceiling_conflict() {
    let facts = vec![
        make_fact("f1", "Vegetarian", RuleConstraintType::BaselineRule, "diet"),
        make_fact("f2", "peanut", RuleConstraintType::PermittedException, "diet_exc"),
        make_fact("f3", "peanut", RuleConstraintType::SafetyCeiling, "allergy"),
    ];

    let advice = resolve_advice(&facts, "Peanut Butter Toast", &["peanut"]);

    // Safety ceiling must ALWAYS win
    assert!(
        matches!(advice.evaluation, ActionEvaluation::BlockedBySafetyCeiling { .. }),
        "Safety ceiling MUST override exception for same ingredient. Got: {:?}",
        advice.evaluation
    );

    // Check directives: should NOT present conflicting instructions to the downstream LLM
    let directives = &advice.directives;
    println!("Compiled directives:\n{}", directives);

    let has_ceiling_directive = directives.contains("[ALLERGY/SAFETY] peanut");
    let has_exception_directive = directives.contains("[EXCEPTION] Permitted: peanut");

    // If both exist simultaneously in compiled directives, it produces conflicting text snippets
    assert!(
        !(has_ceiling_directive && has_exception_directive),
        "CRITICAL: Directives contain contradictory snippets: both [ALLERGY/SAFETY] peanut AND [EXCEPTION] Permitted: peanut!\n{}",
        directives
    );
}

/// Adversarial Test: Multi-ingredient foods where first detected indicator has an exception
/// but second indicator is forbidden.
/// Example: User is Vegetarian with Fish exception (Pescatarian).
/// Food contains BOTH Fish AND Chicken (tags: ["fish", "chicken"]).
#[test]
fn test_multi_meat_where_first_has_exception_and_second_is_blocked() {
    let facts = vec![
        make_fact("f1", "Vegetarian", RuleConstraintType::BaselineRule, "diet"),
        make_fact("f2", "fish", RuleConstraintType::PermittedException, "diet_exc"),
    ];

    // Tags ordered: ["fish", "chicken"]
    let surf_and_turf = resolve_advice(&facts, "Seafood and Chicken Paella", &["fish", "chicken"]);
    assert!(
        matches!(surf_and_turf.evaluation, ActionEvaluation::BlockedByRule { .. }),
        "CRITICAL BUG: Vegetarian with fish exception was PERMITTED chicken because 'fish' appeared first in tags! Got: {:?}",
        surf_and_turf.evaluation
    );
}

/// Adversarial Test: Vegan with Egg exception.
/// Food contains BOTH Eggs AND Chicken (tags: ["eggs", "chicken"]).
#[test]
fn test_vegan_with_egg_exception_and_chicken() {
    let facts = vec![
        make_fact("f1", "Vegan", RuleConstraintType::BaselineRule, "diet"),
        make_fact("f2", "eggs", RuleConstraintType::PermittedException, "diet_exc"),
    ];

    // Tags ordered: ["eggs", "chicken"]
    let egg_chicken = resolve_advice(&facts, "Egg and Chicken Scramble", &["eggs", "chicken"]);
    assert!(
        matches!(egg_chicken.evaluation, ActionEvaluation::BlockedByRule { .. }),
        "CRITICAL BUG: Vegan with egg exception was PERMITTED chicken because 'eggs' appeared first in tags! Got: {:?}",
        egg_chicken.evaluation
    );
}

/// Adversarial Test: Preference is treated as soft preference, never as allergy or hard block
#[test]
fn test_preference_never_causes_hard_block() {
    let facts = vec![
        make_fact("f1", "low dairy", RuleConstraintType::Preference, "dairy_pref"),
        make_fact("f2", "no spicy", RuleConstraintType::Preference, "spice_pref"),
    ];

    let full_dairy = resolve_advice(&facts, "Full Cream Milk", &["dairy", "milk"]);
    assert!(
        matches!(full_dairy.evaluation, ActionEvaluation::Permitted { .. }),
        "Preference must NOT hard block. Got: {:?}",
        full_dairy.evaluation
    );

    if let ActionEvaluation::Permitted { recommendations } = full_dairy.evaluation {
        assert_eq!(recommendations.len(), 2);
        assert!(recommendations.iter().any(|r| r.contains("low dairy")));
    }
}

/// Adversarial Test: Safety ceiling claim is phrased naturally as "peanut allergy"
/// or "severe peanut allergy", but candidate food has tag "peanut" or item is "Peanut Butter".
#[test]
fn test_safety_ceiling_natural_language_claim_matching() {
    let facts = vec![
        make_fact("f1", "peanut allergy", RuleConstraintType::SafetyCeiling, "allergy"),
    ];

    // Candidate is Peanut Butter, tag is "peanut"
    let peanut_butter = resolve_advice(&facts, "Peanut Butter Toast", &["peanut"]);
    assert!(
        matches!(peanut_butter.evaluation, ActionEvaluation::BlockedBySafetyCeiling { .. }),
        "BUG: SafetyCeiling with claim 'peanut allergy' failed to block food with tag 'peanut' and name 'Peanut Butter Toast'! Got: {:?}",
        peanut_butter.evaluation
    );

    // Candidate is Pad Thai with tag "peanuts" (plural)
    let pad_thai = resolve_advice(&facts, "Pad Thai", &["peanuts"]);
    assert!(
        matches!(pad_thai.evaluation, ActionEvaluation::BlockedBySafetyCeiling { .. }),
        "BUG: SafetyCeiling failed to match plural tag 'peanuts' on item 'Pad Thai'! Got: {:?}",
        pad_thai.evaluation
    );
}

/// Adversarial Test: Untagged candidate item with multiple meats in name where earlier indicator in array has exception.
/// Example: Vegetarian with Chicken exception (Pollotarian).
/// Item name: "Chicken and Beef Sausage" (no tags provided).
#[test]
fn test_name_based_multi_meat_bypass() {
    let facts = vec![
        make_fact("f1", "Vegetarian", RuleConstraintType::BaselineRule, "diet"),
        make_fact("f2", "chicken", RuleConstraintType::PermittedException, "diet_exc"),
    ];

    // No tags, only name. "chicken" appears before "beef" in meat_indicators slice.
    let chicken_beef = resolve_advice(&facts, "Chicken and Beef Sausage", &[]);
    assert!(
        matches!(chicken_beef.evaluation, ActionEvaluation::BlockedByRule { .. }),
        "CRITICAL BUG: Vegetarian with chicken exception was PERMITTED beef in 'Chicken and Beef Sausage' because chicken matched first! Got: {:?}",
        chicken_beef.evaluation
    );
}


