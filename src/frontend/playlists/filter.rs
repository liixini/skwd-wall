use crate::contracts::playlists::playlist_source::{ALL, clauses, folder_clause, folder_path};

pub fn source_colors(source: &str) -> Vec<i64> {
    clauses(source)
        .iter()
        .find_map(|clause| {
            let rest = clause.strip_prefix("color:").or_else(|| clause.strip_prefix("colour:"))?;
            Some(
                rest.split(',')
                    .filter_map(|token| crate::frontend::ui::parse_color_bucket(token.trim()))
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default()
}

pub fn toggle_color_clause(source: &str, bucket: i64) -> String {
    let mut clauses = clauses(source);
    let current = clauses
        .iter()
        .position(|clause| clause.starts_with("color:") || clause.starts_with("colour:"));
    let mut buckets: Vec<i64> = current
        .and_then(|index| clauses[index].split_once(':').map(|(_, value)| value))
        .map(|value| {
            value
                .split(',')
                .filter_map(|token| crate::frontend::ui::parse_color_bucket(token.trim()))
                .collect()
        })
        .unwrap_or_default();
    if let Some(position) = buckets.iter().position(|current| *current == bucket) {
        buckets.remove(position);
    } else {
        buckets.push(bucket);
    }
    if let Some(index) = current {
        clauses.remove(index);
    }
    if !buckets.is_empty() {
        let names: Vec<&str> =
            buckets.iter().map(|value| crate::frontend::ui::color_bucket_name(*value)).collect();
        clauses.push(format!("color:{}", names.join(",")));
    }
    clauses.join(" ")
}

pub fn source_folders(source: &str) -> Vec<String> {
    clauses(source)
        .iter()
        .filter_map(|clause| folder_path(clause))
        .map(|folder| trimmed(&folder).to_owned())
        .collect()
}

fn trimmed(folder: &str) -> &str {
    let trimmed = folder.trim_end_matches('/');
    if trimmed.is_empty() { folder } else { trimmed }
}

pub fn toggle_folder_clause(source: &str, folder: &str) -> String {
    let mut clauses = clauses(source);
    let before = clauses.len();
    clauses
        .retain(|clause| folder_path(clause).is_none_or(|path| trimmed(&path) != trimmed(folder)));
    if clauses.len() == before {
        clauses.retain(|clause| clause != ALL);
        clauses.push(folder_clause(folder));
    }
    if clauses.is_empty() {
        return ALL.to_owned();
    }
    clauses.join(" ")
}
