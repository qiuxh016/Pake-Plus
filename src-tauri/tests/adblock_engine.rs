use app_lib::adblock::engine::AdblockEngine;
use app_lib::adblock::BUILTIN_RULES;

#[test]
fn blocks_known_ad_domain() {
    let engine = AdblockEngine::from_rules_text(BUILTIN_RULES, "");
    assert!(engine.should_block_url("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js"));
}

#[test]
fn allows_normal_domain() {
    let engine = AdblockEngine::from_rules_text(BUILTIN_RULES, "");
    assert!(!engine.should_block_url("https://example.com/article"));
}

#[test]
fn custom_rule_is_merged() {
    let engine = AdblockEngine::from_rules_text(BUILTIN_RULES, "||evil-tracker.test^");
    assert!(engine.should_block_url("https://evil-tracker.test/pixel"));
}
