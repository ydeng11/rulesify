pub mod executor;
pub mod instructions;
pub mod tool_paths;

#[cfg(test)]
mod executor_tests;
#[cfg(test)]
mod instructions_tests;
#[cfg(test)]
mod tool_paths_tests;

pub use executor::{
    execute_npx_install, execute_npx_uninstall, install_mega_skill, install_skill,
    parse_source_url, print_install_summary, print_uninstall_summary, uninstall_skill,
    InstallResult, UninstallResult,
};
pub use instructions::{
    generate_install_instructions, generate_instructions, generate_uninstall_instructions,
    generate_uninstall_instructions_batch,
};
pub use tool_paths::{get_skill_folder, get_skill_path};
