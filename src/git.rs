use crate::utils;

// NOTE: These are blocking function calls and are being called in an async context. But it is
// OK cause this is client code anyway.
pub fn is_git(normalised_path: &utils::NormalisedPath) -> bool {
    let mut path = normalised_path.path();
    if path.join(".git").exists() {
        return true;
    }
    while let Some(parent) = path.parent() {
        if parent.join(".git").exists() {
            return true;
        }
        path = parent;
    }
    false
}
