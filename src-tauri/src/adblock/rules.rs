use regex::Regex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum NetworkRule {
    Domain(String),
    UrlRegex(Arc<Regex>),
}

#[derive(Debug, Clone)]
pub struct CosmeticRule {
    pub selector: String,
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct ParsedRules {
    pub network: Vec<NetworkRule>,
    pub cosmetic: Vec<CosmeticRule>,
}

fn normalize_domain(domain: &str) -> String {
    domain.trim_start_matches('.').to_ascii_lowercase()
}

fn parse_network_rule(line: &str) -> Option<NetworkRule> {
    let line = line.trim();
    if line.starts_with("||") {
        let rest = line.strip_prefix("||")?;
        let domain = rest.split('^').next()?.split('/').next()?.trim();
        if domain.is_empty() {
            return None;
        }
        return Some(NetworkRule::Domain(normalize_domain(domain)));
    }

    if line.starts_with('/') && line.ends_with('/') && line.len() > 2 {
        let pattern = &line[1..line.len() - 1];
        let regex = Regex::new(pattern).ok()?;
        return Some(NetworkRule::UrlRegex(Arc::new(regex)));
    }

    None
}

fn parse_cosmetic_rule(line: &str) -> Option<CosmeticRule> {
    let line = line.trim();
    let (domains_part, selector) = if let Some((left, right)) = line.split_once("##") {
        if left.is_empty() {
            (None, format!("##{right}"))
        } else {
            let domains = left
                .split(',')
                .map(normalize_domain)
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>();
            let selector = if right.starts_with('#') {
                format!("###{}", right.trim_start_matches('#'))
            } else {
                format!("##{right}")
            };
            (Some(domains), selector)
        }
    } else if let Some((left, right)) = line.split_once("###") {
        if left.is_empty() {
            (None, format!("###{right}"))
        } else {
            let domains = left
                .split(',')
                .map(normalize_domain)
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>();
            (Some(domains), format!("###{right}"))
        }
    } else {
        return None;
    };

    if selector.len() <= 2 {
        return None;
    }

    Some(CosmeticRule {
        selector,
        domains: domains_part,
    })
}

pub fn parse_rules_text(text: &str) -> ParsedRules {
    let mut parsed = ParsedRules::default();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
            continue;
        }

        if line.contains("##") || line.contains("###") {
            if let Some(rule) = parse_cosmetic_rule(line) {
                parsed.cosmetic.push(rule);
            }
            continue;
        }

        if let Some(rule) = parse_network_rule(line) {
            parsed.network.push(rule);
        }
    }

    parsed
}

pub fn parse_custom_rule(line: &str) -> Option<ParsedRules> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let mut parsed = ParsedRules::default();
    if line.contains("##") || line.contains("###") {
        parsed.cosmetic = parse_cosmetic_rule(line).into_iter().collect();
    } else if let Some(rule) = parse_network_rule(line) {
        parsed.network.push(rule);
    } else {
        return None;
    }

    Some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_domain_rule() {
        let rules = parse_rules_text("||doubleclick.net^");
        assert_eq!(rules.network.len(), 1);
        match &rules.network[0] {
            NetworkRule::Domain(d) => assert_eq!(d, "doubleclick.net"),
            _ => panic!("expected domain rule"),
        }
    }

    #[test]
    fn parses_regex_rule() {
        let rules = parse_rules_text("/ads\\.js/");
        assert_eq!(rules.network.len(), 1);
        assert!(matches!(rules.network[0], NetworkRule::UrlRegex(_)));
    }

    #[test]
    fn parses_cosmetic_rule() {
        let rules = parse_rules_text("##.adsbygoogle");
        assert_eq!(rules.cosmetic.len(), 1);
        assert_eq!(rules.cosmetic[0].selector, "##.adsbygoogle");
    }

    #[test]
    fn parses_domain_specific_cosmetic_rule() {
        let rules = parse_rules_text("example.com##.popup-ad");
        assert_eq!(rules.cosmetic.len(), 1);
        assert_eq!(rules.cosmetic[0].selector, "##.popup-ad");
        assert_eq!(
            rules.cosmetic[0].domains,
            Some(vec!["example.com".to_string()])
        );
    }
}
