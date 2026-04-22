use std::process::Command;

pub fn check_dependency(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_all_dependencies(deps: &[String]) -> Vec<String> {
    deps.iter()
        .filter(|d| !check_dependency(d))
        .cloned()
        .collect()
}

pub fn check_npx_available() -> bool {
    check_dependency("npx")
}

pub fn check_node_available() -> bool {
    check_dependency("node")
}
