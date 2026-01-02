use crate::game_launcher::models::RuleContext;
use crate::game_launcher::rule::should_apply_rules;
use crate::game_resolver::models::Library;
use crate::utils::abs_path_buf::AbsPathBuf;

impl Library {
    pub fn to_classpath_entry(
        &self,
        libraries_dir: AbsPathBuf,
        rule_context: RuleContext,
    ) -> Option<AbsPathBuf> {
        if self.natives.is_some() {
            return None;
        }
        if let Some(rules) = self.rules.clone()
            && !should_apply_rules(rules, rule_context)
        {
            return None;
        }
        self.get_jar_path(libraries_dir)
    }

    pub fn get_jar_path(&self, libraries_dir: AbsPathBuf) -> Option<AbsPathBuf> {
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        let group_id = parts[0];
        let artifact_id = parts[1];
        let version = parts[2];
        let classifier = parts.get(3);

        let mut path = libraries_dir;
        for segment in group_id.split('.') {
            path.push(segment);
        }

        path.push(artifact_id);
        path.push(version);

        let file_name = if let Some(c) = classifier {
            format!("{artifact_id}-{version}-{c}.jar")
        } else {
            format!("{artifact_id}-{version}.jar")
        };

        path.push(file_name);

        Some(path)
    }
}
