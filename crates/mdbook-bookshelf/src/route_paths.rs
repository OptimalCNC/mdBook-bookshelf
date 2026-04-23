use std::path::{Component, Path, PathBuf};

pub(crate) fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let from_components = normalized_components(from_dir);
    let to_components = normalized_components(to_path);
    let common_len = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common_len..] {
        relative.push(component);
    }

    relative
}

pub(crate) fn path_to_string(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    }
}

fn normalized_components(path: &Path) -> Vec<PathBuf> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(PathBuf::from(part)),
            Component::CurDir => None,
            Component::ParentDir => Some(PathBuf::from("..")),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect()
}
