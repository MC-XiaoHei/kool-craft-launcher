use anyhow::{Context, Result};
use fluent_syntax::ast::Entry::Message;
use fluent_syntax::{ast, parser};
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

pub fn get_translate_keys() -> Result<Vec<String>> {
    let source_path = resolve_master_ftl_file()?;
    let file_content = read_file_content(source_path.clone())?;
    let ast = parse_fluent_ast(file_content)?;
    Ok(extract_message_ids(ast))
}

pub fn resolve_locales_dir() -> Result<PathBuf> {
    const LOCALES_DIR: &str = "../locales";
    Ok(current_dir()?.join(LOCALES_DIR))
}

pub fn resolve_master_ftl_file() -> Result<PathBuf> {
    const MASTER_FTL_FILE: &str = "./en-US/main.ftl";
    Ok(resolve_locales_dir()?.join(MASTER_FTL_FILE))
}

pub fn resolve_all_ftl_files() -> Result<Vec<PathBuf>> {
    let root_dir = resolve_locales_dir()?;
    if !root_dir.exists() {
        return Ok(vec![]);
    }

    WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|entry| match entry {
            Err(e) => Some(Err(
                anyhow::Error::new(e).context("Failed to read dir entry")
            )),
            Ok(entry) => {
                let path = entry.path();
                if is_ftl_file(path) {
                    Some(Ok(path.to_path_buf()))
                } else {
                    None
                }
            }
        })
        .collect()
}

fn is_ftl_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match path.extension() {
        Some(ext) => ext == "ftl",
        None => false,
    }
}

fn parse_fluent_ast(content: String) -> Result<ast::Resource<String>> {
    parser::parse(content)
        .map_err(|(res, _err)| anyhow::anyhow!("Failed to parse FTL syntax: {:?}", res))
}

fn extract_message_ids(resource: ast::Resource<String>) -> Vec<String> {
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            Message(msg) => Some(msg.id.name.to_string()),
            _ => None,
        })
        .collect()
}

fn read_file_content(path: PathBuf) -> Result<String> {
    fs::read_to_string(path.clone()).context(format!("Couldn't read file {}", path.display()))
}

fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("Couldn't get current directory")
}
