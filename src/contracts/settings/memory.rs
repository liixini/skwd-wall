use super::keys;

pub fn saved_memory_mib(saved: &str) -> Option<u64> {
    let (section, leaf) = keys::drawing::MEMORY_LIMIT_MIB.split_once('.')?;
    skwd_config::drawing_memory_limit_mib(&serde_json::json!({ section: { leaf: saved } }))
}
