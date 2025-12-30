use crate::game_launcher::model::RuleContext;
use crate::game_resolver::model::Rule;
use os_info::{Info, Type};
use regex::Regex;
use std::collections::HashMap;

pub fn should_apply_rules(rules: Vec<Rule>, context: RuleContext) -> bool {
    let mut should_apply = false;
    for rule in rules {
        if rule.should_apply(context.clone()) && !should_apply {
            should_apply = true;
        }
        if !rule.should_apply(context.clone()) {
            return false;
        }
    }
    should_apply
}

impl Rule {
    pub fn should_apply(&self, context: RuleContext) -> bool {
        if self.is_match(context.clone()) {
            self.action == "allow"
        } else {
            false
        }
    }

    fn is_match(&self, context: RuleContext) -> bool {
        self.is_os_compatible(&context.os_info) && self.is_feature_supported(&context.user_features)
    }

    fn is_os_compatible(&self, os_info: &Info) -> bool {
        self.os.as_ref().is_none_or(|os_cond| {
            self.match_name(os_cond.name.clone(), os_info)
                && self.match_arch(os_cond.arch.clone(), os_info)
                && self.match_version(os_cond.version_regex.clone(), os_info)
        })
    }

    fn match_name(&self, os_name: Option<String>, info: &Info) -> bool {
        os_name.is_none_or(|required| required == self.get_platform_name(info.os_type()))
    }

    fn match_arch(&self, os_arch: Option<String>, info: &Info) -> bool {
        os_arch.is_none_or(|required| {
            let current = info.architecture().unwrap_or("unknown").to_lowercase();
            match required.as_str() {
                "x64" | "amd64" => current.contains("x86_64") || current.contains("amd64"),
                "x86" => current.contains("x86") || current.contains("i386"),
                "arm64" | "aarch64" => current.contains("arm64") || current.contains("aarch64"),
                _ => current.contains(&required),
            }
        })
    }

    fn match_version(&self, os_version_regex: Option<String>, info: &Info) -> bool {
        os_version_regex.is_none_or(|pattern| {
            let current = info.version().to_string();
            Regex::new(&pattern).is_ok_and(|re| re.is_match(&current))
        })
    }

    fn get_platform_name(&self, os_type: Type) -> &'static str {
        match os_type {
            Type::Linux => "linux",
            Type::Windows => "windows",
            Type::Macos => "osx",
            _ => "unknown",
        }
    }

    fn is_feature_supported(&self, user_features: &HashMap<String, bool>) -> bool {
        self.features.as_ref().is_none_or(|required| {
            required
                .iter()
                .all(|(k, &v)| user_features.get(k).copied().unwrap_or(false) == v)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_resolver::model::OsCondition;
    use os_info::{Info, Type};
    use serde_json::json;

    fn mock_info(os_type: Type, version: impl Into<String>, arch: impl Into<String>) -> Info {
        let v = json!({
            "os_type": os_type,
            "version": {
                "Custom": version.into(),
            },
            "bitness": "Unknown",
            "architecture": arch.into(),
        });
        serde_json::from_value(v).expect("Failed to deserialize Info mock")
    }

    #[test]
    fn test_os_name_match() {
        let rule = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                name: Some("windows".into()),
                ..Default::default()
            }),
            features: None,
        };

        let win_info = mock_info(Type::Windows, "10.0", "x86_64");
        let linux_info = mock_info(Type::Linux, "5.15", "x86_64");

        assert!(rule.should_apply(RuleContext {
            os_info: win_info,
            user_features: HashMap::new(),
        }));
        assert!(!rule.should_apply(RuleContext {
            os_info: linux_info,
            user_features: HashMap::new(),
        }));
    }

    #[test]
    fn test_version_regex_match() {
        let rule = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                version_regex: Some(r"^10\.".into()),
                ..Default::default()
            }),
            features: None,
        };

        let win10 = mock_info(Type::Windows, "10.0.19045", "x86_64");
        let win11 = mock_info(Type::Windows, "11.0.22621", "x86_64");

        assert!(rule.should_apply(RuleContext {
            os_info: win10,
            user_features: HashMap::new(),
        }));
        assert!(!rule.should_apply(RuleContext {
            os_info: win11,
            user_features: HashMap::new(),
        }));
    }

    #[test]
    fn test_arch_variations() {
        let rule_x64 = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("x64".into()),
                ..Default::default()
            }),
            features: None,
        };

        assert!(rule_x64.should_apply(RuleContext {
            os_info: mock_info(Type::Linux, "5.0", "amd64"),
            user_features: HashMap::new(),
        }));

        let rule_amd64 = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("amd64".into()),
                ..Default::default()
            }),
            features: None,
        };
        assert!(rule_amd64.should_apply(RuleContext {
            os_info: mock_info(Type::Linux, "5.0", "x86_64"),
            user_features: HashMap::new(),
        }));

        let rule_x86 = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("x86".into()),
                ..Default::default()
            }),
            features: None,
        };
        assert!(rule_x86.should_apply(RuleContext {
            os_info: mock_info(Type::Windows, "10.0", "i386"),
            user_features: HashMap::new(),
        }));
        assert!(rule_x86.should_apply(RuleContext {
            os_info: mock_info(Type::Windows, "10.0", "x86"),
            user_features: HashMap::new(),
        }));

        let rule_arm = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("arm64".into()),
                ..Default::default()
            }),
            features: None,
        };
        assert!(rule_arm.should_apply(RuleContext {
            os_info: mock_info(Type::Macos, "14.0", "aarch64"),
            user_features: HashMap::new(),
        }));
        assert!(rule_arm.should_apply(RuleContext {
            os_info: mock_info(Type::Macos, "14.0", "arm64"),
            user_features: HashMap::new(),
        }));

        let rule_aarch = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("aarch64".into()),
                ..Default::default()
            }),
            features: None,
        };
        assert!(rule_aarch.should_apply(RuleContext {
            os_info: mock_info(Type::Macos, "14.0", "aarch64"),
            user_features: HashMap::new(),
        }));
        assert!(rule_aarch.should_apply(RuleContext {
            os_info: mock_info(Type::Macos, "14.0", "arm64"),
            user_features: HashMap::new(),
        }));

        let rule_not_match = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                arch: Some("wtf".into()),
                ..Default::default()
            }),
            features: None,
        };
        assert!(!rule_not_match.should_apply(RuleContext {
            os_info: mock_info(Type::Macos, "14.0", "arm64"),
            user_features: HashMap::new(),
        }));
    }

    #[test]
    fn test_features_matching() {
        let mut features = HashMap::new();
        features.insert("is_demo_user".into(), true);
        features.insert("has_custom_resolution".into(), false);

        let rule = Rule {
            action: "allow".into(),
            os: None,
            features: Some(features),
        };

        let mut user_f = HashMap::new();
        user_f.insert("is_demo_user".into(), true);
        user_f.insert("has_custom_resolution".into(), false);

        assert!(rule.should_apply(RuleContext {
            os_info: mock_info(Type::Linux, "1.0", "x64"),
            user_features: user_f.clone(),
        }));

        user_f.remove("is_demo_user");
        assert!(!rule.should_apply(RuleContext {
            os_info: mock_info(Type::Linux, "1.0", "x64"),
            user_features: user_f.clone(),
        }));
    }

    #[test]
    fn test_invalid_regex_safety() {
        let rule = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                version_regex: Some(r"*[invalid(".into()),
                ..Default::default()
            }),
            features: None,
        };

        assert!(!rule.should_apply(RuleContext {
            os_info: mock_info(Type::Windows, "10.0", "x64"),
            user_features: HashMap::new(),
        }));
    }

    #[test]
    fn test_combined_restrictions() {
        let mut rf = HashMap::new();
        rf.insert("has_custom_resolution".into(), true);

        let rule = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                name: Some("osx".into()),
                ..Default::default()
            }),
            features: Some(rf),
        };

        let osx_info = mock_info(Type::Macos, "14.0", "x64");
        let mut user_f = HashMap::new();
        user_f.insert("has_custom_resolution".into(), true);

        assert!(rule.should_apply(RuleContext {
            os_info: osx_info.clone(),
            user_features: user_f.clone(),
        }));

        user_f.insert("has_custom_resolution".into(), false);
        assert!(!rule.should_apply(RuleContext {
            os_info: osx_info.clone(),
            user_features: user_f.clone(),
        }));

        user_f.insert("has_custom_resolution".into(), true);
        let win_info = mock_info(Type::Windows, "10.0", "x64");
        assert!(!rule.should_apply(RuleContext {
            os_info: win_info,
            user_features: user_f,
        }));
    }

    #[test]
    fn test_disallow_action() {
        let rule = Rule {
            action: "disallow".into(),
            os: None,
            features: None,
        };

        assert!(!rule.should_apply(RuleContext {
            os_info: mock_info(Type::Linux, "1.0", "x64"),
            user_features: HashMap::new(),
        }))
    }

    #[test]
    fn test_unknown_os_handling() {
        let rule = Rule {
            action: "allow".into(),
            os: Some(OsCondition {
                name: Some("linux".into()),
                ..Default::default()
            }),
            features: None,
        };

        assert!(!rule.should_apply(RuleContext {
            os_info: mock_info(Type::Unknown, "1.0", "x64"),
            user_features: HashMap::new(),
        }))
    }

    #[test]
    fn test_missing_fields_defaults() {
        let rule = Rule {
            action: "allow".into(),
            os: None,
            features: None,
        };

        assert!(rule.should_apply(RuleContext {
            os_info: mock_info(Type::Windows, "10.0", "x64"),
            user_features: HashMap::new(),
        }));
    }
}
