#![cfg_attr(coverage_nightly, coverage(off))]

use crate::config::codegen::{get_config_type_def, get_config_watcher};
use crate::utils::codegen::unwrap_safe::{gen_config_watcher, gen_types};
use specta_typescript::Typescript;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[cfg(debug_assertions)]
pub fn do_codegen() {
    gen_types();
    gen_config_watcher();
}

// use .unwrap() here is safe, because these function only calls in debug env
#[cfg(debug_assertions)]
mod unwrap_safe {
    use super::*;

    pub(super) fn gen_types() {
        let path = "../src/bindings/types.ts";

        Typescript::default()
            .export_to(path, &specta::export())
            .unwrap();

        let mut file = OpenOptions::new().append(true).open(path).unwrap();

        writeln!(file).unwrap();
        writeln!(file, "{}", get_config_type_def()).unwrap();
    }

    pub(super) fn gen_config_watcher() {
        let path = "../src/bindings/config.ts";
        let mut content = get_config_watcher();
        fs::write(path, content).unwrap();
    }
}

pub fn indent_all(text: impl Into<String>, space_of_num: usize) -> String {
    let space = " ".repeat(space_of_num);
    text.into()
        .lines()
        .map(|line| format!("{}{}", space, line))
        .collect::<Vec<_>>()
        .join("\n")
}
