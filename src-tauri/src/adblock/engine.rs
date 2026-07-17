use super::rules::{parse_custom_rule, parse_rules_text, CosmeticRule, NetworkRule, ParsedRules};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use url::Url;

#[derive(Debug, Clone, Serialize)]
pub struct AdblockExport {
    pub domains: Vec<String>,
    pub regexes: Vec<String>,
    pub cosmetic_selectors: Vec<String>,
}

struct EngineRules {
    network: Vec<NetworkRule>,
    cosmetic: Vec<CosmeticRule>,
    custom_lines: Vec<String>,
}

pub struct AdblockEngine {
    rules: RwLock<EngineRules>,
    blocked_count: AtomicU32,
    builtin: String,
}

impl AdblockEngine {
    pub fn from_rules_text(builtin: &str, custom: &str) -> Self {
        let parsed = Self::merge_texts(builtin, custom);
        Self {
            rules: RwLock::new(EngineRules {
                network: parsed.network,
                cosmetic: parsed.cosmetic,
                custom_lines: Vec::new(),
            }),
            blocked_count: AtomicU32::new(0),
            builtin: builtin.to_string(),
        }
    }

    fn merge_texts(builtin: &str, custom: &str) -> ParsedRules {
        let mut parsed = parse_rules_text(builtin);
        let custom_parsed = parse_rules_text(custom);
        parsed.network.extend(custom_parsed.network);
        parsed.cosmetic.extend(custom_parsed.cosmetic);
        parsed
    }

    pub fn rule_count(&self) -> usize {
        let rules = self.rules.read().unwrap_or_else(|e| e.into_inner());
        rules.network.len() + rules.cosmetic.len() + rules.custom_lines.len()
    }

    pub fn reset_page_count(&self) {
        self.blocked_count.store(0, Ordering::Relaxed);
    }

    pub fn blocked_count(&self) -> u32 {
        self.blocked_count.load(Ordering::Relaxed)
    }

    pub fn set_page_count(&self, count: u32) {
        self.blocked_count.store(count, Ordering::Relaxed);
    }

    pub fn record_block(&self) -> u32 {
        self.blocked_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn host_matches_domain(host: &str, domain: &str) -> bool {
        let host = host.to_ascii_lowercase();
        let domain = domain.to_ascii_lowercase();
        host == domain || host.ends_with(&format!(".{domain}"))
    }

    pub fn should_block_url(&self, raw_url: &str) -> bool {
        let rules = self.rules.read().unwrap_or_else(|e| e.into_inner());
        let Ok(url) = Url::parse(raw_url) else {
            return Self::match_regex_only(&rules.network, raw_url);
        };

        let host = match url.host_str() {
            Some(h) => h,
            None => return false,
        };

        let full = url.as_str();
        for rule in &rules.network {
            match rule {
                NetworkRule::Domain(domain) => {
                    if Self::host_matches_domain(host, domain) {
                        return true;
                    }
                }
                NetworkRule::UrlRegex(regex) => {
                    if regex.is_match(full) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn match_regex_only(network: &[NetworkRule], raw_url: &str) -> bool {
        network.iter().any(|rule| {
            if let NetworkRule::UrlRegex(regex) = rule {
                regex.is_match(raw_url)
            } else {
                false
            }
        })
    }

    pub fn check_and_block(&self, raw_url: &str) -> bool {
        if self.should_block_url(raw_url) {
            self.record_block();
            true
        } else {
            false
        }
    }

    pub fn export_for_injection(&self, page_host: Option<&str>) -> AdblockExport {
        let rules = self.rules.read().unwrap_or_else(|e| e.into_inner());
        let mut domains = Vec::new();
        let mut regexes = Vec::new();

        for rule in &rules.network {
            match rule {
                NetworkRule::Domain(domain) => domains.push(domain.clone()),
                NetworkRule::UrlRegex(regex) => regexes.push(regex.as_str().to_string()),
            }
        }

        let cosmetic_selectors = rules
            .cosmetic
            .iter()
            .filter(|rule| Self::cosmetic_applies(rule, page_host))
            .map(|rule| rule.selector.clone())
            .collect();

        AdblockExport {
            domains,
            regexes,
            cosmetic_selectors,
        }
    }

    fn cosmetic_applies(rule: &CosmeticRule, page_host: Option<&str>) -> bool {
        match (&rule.domains, page_host) {
            (None, _) => true,
            (Some(domains), Some(host)) => {
                let host = host.to_ascii_lowercase();
                domains.iter().any(|d| Self::host_matches_domain(&host, d))
            }
            (Some(_), None) => false,
        }
    }

    pub fn add_custom_rule(&self, line: &str) -> Result<(), String> {
        parse_custom_rule(line).ok_or_else(|| "Invalid adblock rule format".to_string())?;
        let mut rules = self
            .rules
            .write()
            .map_err(|_| "Failed to lock adblock rules".to_string())?;
        rules.custom_lines.push(line.trim().to_string());
        let merged = Self::merge_texts(&self.builtin, &rules.custom_lines.join("\n"));
        rules.network = merged.network;
        rules.cosmetic = merged.cosmetic;
        Ok(())
    }

    pub fn remove_custom_rule(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let mut rules = match self.rules.write() {
            Ok(r) => r,
            Err(_) => return false,
        };
        let before = rules.custom_lines.len();
        rules.custom_lines.retain(|r| r != trimmed);
        if rules.custom_lines.len() == before {
            return false;
        }
        let merged = Self::merge_texts(&self.builtin, &rules.custom_lines.join("\n"));
        rules.network = merged.network;
        rules.cosmetic = merged.cosmetic;
        true
    }

    pub fn custom_rules(&self) -> Vec<String> {
        self.rules
            .read()
            .map(|r| r.custom_lines.clone())
            .unwrap_or_default()
    }
}
