#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use kool_craft_launcher_lib::utils::codegen::do_codegen;

fn main() -> anyhow::Result<()> {
    do_codegen()
}
