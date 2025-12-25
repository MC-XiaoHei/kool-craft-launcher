use crate::launcher::model::{ArgumentsContext, ArgumentsInfo, RuleContext};
use crate::launcher::rule::should_apply_rules;
use crate::resolver::model::{ArgumentValue, Arguments};
use std::collections::HashSet;
use tap::{Pipe, Tap};

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
        Self::deduplicate_args(args)
    }

    pub fn get_jvm_arguments(
        &self,
        context: RuleContext,
        metadata: ArgumentsContext,
        custom_jvm_arguments: Vec<String>,
    ) -> Vec<String> {
        let mut args = match self {
            ArgumentsInfo::Legacy(_) => vec![], // TODO
            ArgumentsInfo::Modern(args) => args.get_raw_jvm_arguments(context),
        };
        args = metadata.replace_args_placeholders(args);
        args.append(&mut custom_jvm_arguments.clone());
        Self::deduplicate_args(args)
    }

    fn deduplicate_args(args: Vec<String>) -> Vec<String> {
        const ARG_PREFIX: char = '-';
        let mut seen = HashSet::new();
        let mut result = Vec::with_capacity(args.len());

        let mut iter = args.into_iter().rev().peekable();

        while let Some(arg) = iter.next() {
            let is_pair_value = !arg.starts_with(ARG_PREFIX)
                && iter.peek().is_some_and(|k| k.starts_with(ARG_PREFIX));

            if is_pair_value {
                if let Some(key) = iter.next()
                    && seen.insert(key.clone())
                {
                    result.push(arg);
                    result.push(key);
                }
            } else if seen.insert(arg.clone()) {
                result.push(arg);
            }
        }

        result.reverse();
        result
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
            ArgumentValue::Complex { value, rules } => {
                if should_apply_rules(rules.clone(), context.clone()) {
                    value.clone().into_vec()
                } else {
                    vec![]
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_dedupe_mixed_prefixes() {
        #[rustfmt::skip]
        let input = vec![
            "-Xmx".into(), "1G".into(),
            "-Xmx".into(), "2G".into(),
            "--username".into(), "Steve".into(),
            "--username".into(), "Alex".into(),
        ];
        let output = ArgumentsInfo::deduplicate_args(input);
        #[rustfmt::skip]
        assert_eq!(output, vec![
            "-Xmx".to_string(), "2G".into(),
            "--username".into(), "Alex".into()
        ]);
    }

    #[test]
    fn test_flags_and_pairs() {
        #[rustfmt::skip]
        let input = vec![
            "--fullscreen".into(),
            "--width".into(), "800".into(),
            "--fullscreen".into(),
            "--width".into(), "1024".into(),
        ];
        let output = ArgumentsInfo::deduplicate_args(input);
        #[rustfmt::skip]
        assert_eq!(output, vec![
            "--fullscreen".to_string(),
            "--width".into(), "1024".into()
        ]);
    }

    #[test]
    fn test_path_with_hyphen() {
        #[rustfmt::skip]
        let input = vec![
            "--gameDir".into(), "my-mc-folder".into(),
            "--gameDir".into(), "final-folder".into(),
        ];
        let output = ArgumentsInfo::deduplicate_args(input);
        assert_eq!(output, vec!["--gameDir".to_string(), "final-folder".into()]);
    }

    #[test]
    fn test_not_a_pair_if_next_is_key() {
        #[rustfmt::skip]
        let input = vec![
            "-someFlag".into(),
            "-otherFlag".into(),
        ];
        let output = ArgumentsInfo::deduplicate_args(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn test_mixed_complex_args() {
        #[rustfmt::skip]
        let input = vec![
            "-Xmx".into(), "1G".into(),
            "-Dminecraft.launcher.brand=hmcl".into(),
            "-Xmx".into(), "2G".into(),
            "--username".into(), "Steve".into(),
            "--username".into(), "Alex".into(),
        ];
        let output = ArgumentsInfo::deduplicate_args(input);

        #[rustfmt::skip]
        assert_eq!(output, vec![
            "-Dminecraft.launcher.brand=hmcl".to_string(),
            "-Xmx".into(), "2G".into(),
            "--username".into(), "Alex".into(),
        ]);
    }
}
