use crate::launcher::rule::{RuleContext, should_apply_rules};
use crate::resolver::model::{ArgumentValue, Arguments};
use std::collections::HashMap;

impl Arguments {
    pub fn get_game_arguments(
        &self,
        context: RuleContext,
        launch_args: HashMap<String, String>,
    ) -> Vec<String> {
        Self::get_arguments(self.game.clone(), context, &launch_args)
    }

    pub fn get_jvm_arguments(
        &self,
        context: RuleContext,
        launch_args: HashMap<String, String>,
    ) -> Vec<String> {
        Self::get_arguments(self.jvm.clone(), context, &launch_args)
    }

    fn get_arguments(
        args: Vec<ArgumentValue>,
        context: RuleContext,
        launch_args: &HashMap<String, String>,
    ) -> Vec<String> {
        args.iter()
            .fold(vec![], |mut acc, arg| {
                acc.extend(arg.get_value(context.clone()));
                acc
            })
            .iter()
            .map(|arg| Self::replace_placeholder(arg.into(), launch_args))
            .collect()
    }

    fn replace_placeholder(arg: String, launch_args: &HashMap<String, String>) -> String {
        for (key, value) in launch_args {
            let placeholder = format!("${{{key}}}");
            if arg == placeholder {
                return value.into();
            }
        }
        arg
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
    use crate::resolver::model::Arguments;
    use std::collections::HashMap;

    #[test]
    fn test_placeholder_replacement() {
        let value = "test_value".to_string();
        let launch_args = HashMap::from([("placeholder".to_string(), value.clone())]);

        let result = Arguments::replace_placeholder("${placeholder}".into(), &launch_args);
        assert_eq!(
            result,
            value.clone(),
            "Placeholder should be replaced with the correct value"
        );

        let no_replace = "no_replacement_needed";
        let result_no_replace = Arguments::replace_placeholder(no_replace.into(), &launch_args);
        assert_eq!(
            result_no_replace, no_replace,
            "String without placeholder should remain unchanged"
        );
    }
}
