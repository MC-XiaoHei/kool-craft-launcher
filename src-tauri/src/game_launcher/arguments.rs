use crate::game_launcher::model::{ArgumentsContext, ArgumentsInfo, RuleContext};
use crate::game_launcher::rule::should_apply_rules;
use crate::game_resolver::model::{ArgumentValue, Arguments, OsCondition, Rule};
use ArgumentValue::Complex;

impl ArgumentsInfo {
    pub fn get_game_arguments(
        &self,
        context: RuleContext,
        metadata: ArgumentsContext,
        custom_game_arguments: Vec<String>,
    ) -> Vec<String> {
        let mut args = match self {
            ArgumentsInfo::Legacy(game_args) => game_args
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            ArgumentsInfo::Modern(args) => args.get_raw_game_arguments(context),
        };
        args = metadata.replace_args_placeholders(args);
        args.append(&mut custom_game_arguments.clone());
        args
    }

    pub fn get_jvm_arguments(
        &self,
        context: RuleContext,
        metadata: ArgumentsContext,
        custom_jvm_arguments: Vec<String>,
    ) -> Vec<String> {
        let mut args = match self {
            ArgumentsInfo::Legacy(_) => Self::get_legacy_jvm_args(context),
            ArgumentsInfo::Modern(args) => args.get_raw_jvm_arguments(context),
        };
        args = metadata.replace_args_placeholders(args);
        args.append(&mut custom_jvm_arguments.clone());
        args
    }

    fn get_legacy_jvm_args(context: RuleContext) -> Vec<String> {
        let args: Vec<ArgumentValue> = vec![
            Complex {
                rules: vec![Rule {
                    action: "allow".into(),
                    os: Some(OsCondition {
                        name: Some("osx".into()),
                        version_regex: None,
                        arch: None,
                    }),
                    features: None,
                }],
                value: "-XstartOnFirstThread".into(),
            },
            Complex {
                rules: vec![Rule {
                    action: "allow".into(),
                    os: Some(OsCondition {
                        name: Some("windows".into()),
                        version_regex: Some("^10\\.".into()),
                        arch: None,
                    }),
                    features: None,
                }],
                value: vec!["-Dos.name=Windows 10", "-Dos.version=10.0"].into(),
            },
            Complex {
                rules: vec![Rule {
                    action: "allow".into(),
                    os: Some(OsCondition {
                        name: None,
                        version_regex: None,
                        arch: Some("x86".into()),
                    }),
                    features: None,
                }],
                value: "-Xss1M".into(),
            },
            "-Djava.library.path=${natives_directory}".into(),
            "-Dminecraft.launcher.brand=${launcher_name}".into(),
            "-Dminecraft.launcher.version=${launcher_version}".into(),
            "-cp".into(),
            "${classpath}".into(),
        ];
        Arguments {
            game: vec![],
            jvm: args,
        }
        .get_raw_jvm_arguments(context)
    }
}

impl Arguments {
    pub fn get_raw_game_arguments(&self, context: RuleContext) -> Vec<String> {
        Self::get_raw_arguments(self.game.clone(), context)
    }

    pub fn get_raw_jvm_arguments(&self, context: RuleContext) -> Vec<String> {
        Self::get_raw_arguments(self.jvm.clone(), context)
    }

    fn get_raw_arguments(args: Vec<ArgumentValue>, context: RuleContext) -> Vec<String> {
        args.iter()
            .fold(vec![], |mut acc, arg| {
                acc.extend(arg.get_value(context.clone()));
                acc
            })
            .into_iter()
            .collect()
    }
}

impl ArgumentValue {
    pub fn get_value(&self, context: RuleContext) -> Vec<String> {
        match self {
            ArgumentValue::Simple(simple) => vec![simple.into()],
            Complex { value, rules } => {
                if should_apply_rules(rules.clone(), context.clone()) {
                    value.clone().into_vec()
                } else {
                    vec![]
                }
            }
        }
    }
}
