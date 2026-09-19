use std::collections::HashMap;

use crate::fetch_util::{Config, Solution, Spec, SpecOrAlias};

pub struct Chromium;

impl Config for Chromium {
    fn fetch_spec(&self, props: &HashMap<String, String>) -> SpecOrAlias {
        let url = "https://chromium.googlesource.com/chromium/src.git".to_string();

        let mut solution = Solution {
            name: "src".to_string(),
            url,
            deps_file: "DEPS".to_string(),
            ..Default::default()
        };

        if props.get("webkit_revision").is_some_and(|s| s == "ToT") {
            solution
                .custom_vars
                .insert("webkit_revision".into(), "".into());
        }
        if props.get("internal").is_some_and(|s| s == "True") {
            solution
                .custom_vars
                // XXX: bool literal true may be better here
                .insert("checkout_src_internal".to_string(), "True".into());
        }

        let mut spec = Spec {
            solutions: vec![solution],
            ..Default::default()
        };

        if let Some(target_os) = props.get("target_os") {
            spec.target_os
                .extend(target_os.split(",").map(|s| s.to_string()));
        }
        if let Some(target_os_only) = props.get("target_os_only") {
            spec.target_os_only = target_os_only.clone();
        }

        SpecOrAlias::Left(spec)
    }

    fn expected_roots(&self, _props: &HashMap<String, String>) -> String {
        "src".to_string()
    }
}
