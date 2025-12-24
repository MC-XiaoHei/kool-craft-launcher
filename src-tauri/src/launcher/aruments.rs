use crate::launcher::rule::{RuleContext, should_apply_rules};
use crate::resolver::model::{ArgumentValue, Arguments};
use std::collections::HashMap;

impl Arguments {
    pub fn get_game_arguments(
        &self,
        context: RuleContext,
        launch_args: HashMap<String, String>,
    ) -> Vec<String> {
        Self::get_arguments(self.game.clone(), context, launch_args)
    }

    pub fn get_jvm_arguments(
        &self,
        context: RuleContext,
        launch_args: HashMap<String, String>,
    ) -> Vec<String> {
        Self::get_arguments(self.jvm.clone(), context, launch_args)
    }

    fn get_arguments(
        args: Vec<ArgumentValue>,
        context: RuleContext,
        launch_args: HashMap<String, String>,
    ) -> Vec<String> {
        let mut result = vec![];
        for arg in args {
            let mut value = arg.get_value(context.clone());
            for v in value.iter_mut() {
                for (key, val) in launch_args.iter() {
                    if v == format!("${{{}}}", key).as_str() {
                        *v = val.to_string();
                        break;
                    }
                }
            }
            result.extend(value);
        }
        result
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
