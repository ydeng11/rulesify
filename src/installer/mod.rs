pub mod instructions;
pub mod tool_paths;

#[cfg(test)]
mod instructions_tests;
#[cfg(test)]
mod tool_paths_tests;

pub use instructions::{
    generate_install_instructions, generate_instructions, generate_uninstall_instructions,
};
pub use tool_paths::{get_skill_folder, get_skill_path};
