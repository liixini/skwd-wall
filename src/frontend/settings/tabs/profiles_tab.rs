use super::{Builder, keys};
use crate::contracts::settings::{EVERYTHING_PROFILE, ProfileState, SettingsSource, folder_tag};
use crate::frontend::settings::{ProfileEntry, ProfilesPage, TreeNode};
use crate::i18n::{tr, tr_args};

pub const STATES: [(ProfileState, &str); 3] = [
    (ProfileState::Show, "settings-profiles-state-show"),
    (ProfileState::Hide, "settings-profiles-state-hide"),
    (ProfileState::Exclude, "settings-profiles-state-exclude"),
];
pub const FOLDER_SEGMENT: &str = "folders";
pub const FOLD_PREFIX: &str = "fold";
pub const SEGMENT_PREFIX: &str = "seg:";
pub const WORKSHOP_TAG: &str = "workshop";
const HELM_PROFILE: &str = "skwd-helm profile";
const WALL_PROFILE: &str = "skwd-wall-v2 --profile";

pub fn profile_commands(id: &str) -> [String; 2] {
    [format!("{HELM_PROFILE} {id}"), format!("{WALL_PROFILE} {id}")]
}

pub(super) fn tab_profiles(builder: &mut Builder<'_>) {
    builder.card(tr("settings-profiles-card"), tr("settings-profiles-card-desc"));
}

pub fn profiles_page(
    cfg: &dyn SettingsSource,
    folders: &[String],
    selected: usize,
    filter: &str,
) -> ProfilesPage {
    let active = cfg.text(keys::library::PROFILE);
    let mut profiles = vec![ProfileEntry {
        index: None,
        id: EVERYTHING_PROFILE.to_owned(),
        name: tr("settings-profiles-everything").to_owned(),
        active: active.is_empty() || active == EVERYTHING_PROFILE,
    }];
    profiles.extend((0..cfg.array_len(keys::library::PROFILES)).map(|index| {
        let base = format!("{}.{index}", keys::library::PROFILES);
        let id = cfg.text(&format!("{base}.{}", keys::library::profile::ID));
        let name = cfg.text(&format!("{base}.{}", keys::library::profile::NAME));
        ProfileEntry { active: id == active, index: Some(index), id, name }
    }));
    let selected = selected.min(profiles.len() - 1);
    let entry = &profiles[selected];
    let never_scanned: Vec<String> = (0..cfg.array_len(keys::library::EXCLUDED_FOLDERS))
        .map(|index| cfg.text(&format!("{}.{index}", keys::library::EXCLUDED_FOLDERS)))
        .filter(|folder| !folder.trim().is_empty())
        .collect();
    let (states, workshop) = entry.index.map_or_else(
        || (vec![ProfileState::Show; folders.len()], ProfileState::Show),
        |index| cfg.profile_states(index, folders),
    );
    let mut tree = tree(&entry.id, entry.index, folders, &states, &never_scanned);
    tree.push(TreeNode {
        id: format!("{FOLD_PREFIX}:{}:{WORKSHOP_TAG}", entry.id),
        path: entry.index.map_or_else(String::new, |index| {
            format!("{}.{index}.{}", keys::library::PROFILES, keys::library::profile::WORKSHOP)
        }),
        label: tr("settings-profiles-workshop").to_owned(),
        depth: 0,
        state: Some(workshop),
        mixed: false,
        hidden_inside: 0,
        children: false,
        collapsed: false,
    });
    let query = filter.trim().to_lowercase();
    let no_match = !query.is_empty() && !keep_matches(&mut tree, &query);
    let summary = if entry.builtin() {
        tr("settings-profiles-summary-none").to_owned()
    } else {
        summary(folders, &states, workshop)
    };
    ProfilesPage { profiles, selected, summary, tree, no_match }
}

fn keep_matches(tree: &mut Vec<TreeNode>, query: &str) -> bool {
    let mut keep = vec![false; tree.len()];
    for index in 0..tree.len() {
        if !tree[index].label.to_lowercase().contains(query) {
            continue;
        }
        keep[index] = true;
        let mut depth = tree[index].depth;
        for ancestor in (0..index).rev() {
            if tree[ancestor].depth < depth {
                keep[ancestor] = true;
                depth = tree[ancestor].depth;
            }
        }
    }
    let mut kept = keep.iter();
    tree.retain(|_| *kept.next().unwrap_or(&false));
    for index in 0..tree.len() {
        tree[index].children =
            tree.get(index + 1).is_some_and(|next| next.depth > tree[index].depth);
    }
    !tree.is_empty()
}

struct TreeSource<'a> {
    profile: &'a str,
    index: Option<usize>,
    folders: &'a [String],
    states: &'a [ProfileState],
    parents: Vec<Option<usize>>,
    never_scanned: Vec<String>,
}

fn tree(
    profile: &str,
    index: Option<usize>,
    folders: &[String],
    states: &[ProfileState],
    never_scanned: &[String],
) -> Vec<TreeNode> {
    let parents: Vec<Option<usize>> =
        folders.iter().map(|folder| parent_of(folder, folders)).collect();
    let roots: Vec<usize> =
        (0..folders.len()).filter(|folder| parents[*folder].is_none()).collect();
    let never_scanned = never_scanned
        .iter()
        .flat_map(|entry| {
            if std::path::Path::new(entry).is_absolute() {
                vec![entry.clone()]
            } else {
                roots.iter().map(|root| format!("{}/{entry}", folders[*root])).collect()
            }
        })
        .filter(|entry| !folders.contains(entry))
        .collect();
    let source = TreeSource { profile, index, folders, states, parents, never_scanned };
    let mut nodes = Vec::new();
    for root in roots {
        push_subtree(&mut nodes, &source, root, 0);
    }
    nodes
}

fn push_subtree(nodes: &mut Vec<TreeNode>, source: &TreeSource<'_>, folder: usize, depth: u8) {
    let path = &source.folders[folder];
    let state = source.states[folder];
    let children: Vec<usize> =
        (0..source.folders.len()).filter(|child| source.parents[*child] == Some(folder)).collect();
    let descendants: Vec<usize> =
        (0..source.folders.len()).filter(|other| is_ancestor(source, folder, *other)).collect();
    let never_scanned: Vec<&String> = source
        .never_scanned
        .iter()
        .filter(|entry| parent_of(entry, source.folders) == Some(folder))
        .collect();
    let hidden_inside = descendants
        .iter()
        .filter(|other| {
            source.states[**other] == ProfileState::Hide
                && source.parents[**other]
                    .is_some_and(|parent| source.states[parent] != ProfileState::Hide)
        })
        .count();
    nodes.push(TreeNode {
        id: fold_id(source.profile, path),
        path: source.index.map_or_else(String::new, |index| {
            format!("{}.{index}.{FOLDER_SEGMENT}.{}", keys::library::PROFILES, folder_tag(path))
        }),
        label: if depth == 0 {
            path.clone()
        } else {
            crate::frontend::ui::folder_leaf(path).to_owned()
        },
        depth,
        state: Some(state),
        mixed: descendants.iter().any(|other| source.states[*other] != state),
        hidden_inside,
        children: !children.is_empty() || !never_scanned.is_empty(),
        collapsed: false,
    });
    for child in children {
        push_subtree(nodes, source, child, depth.saturating_add(1));
    }
    for entry in never_scanned {
        nodes.push(TreeNode {
            id: fold_id(source.profile, entry),
            path: String::new(),
            label: std::path::Path::new(entry)
                .strip_prefix(path)
                .map_or_else(|_| entry.clone(), |rest| rest.to_string_lossy().into_owned()),
            depth: depth.saturating_add(1),
            state: None,
            mixed: false,
            hidden_inside: 0,
            children: false,
            collapsed: false,
        });
    }
}

fn fold_id(profile: &str, folder: &str) -> String {
    format!("{FOLD_PREFIX}:{profile}:{}", folder_tag(folder))
}

pub fn segment_key(profile: &str, tag: &str) -> String {
    format!("{SEGMENT_PREFIX}{FOLD_PREFIX}:{profile}:{tag}")
}

fn is_ancestor(source: &TreeSource<'_>, ancestor: usize, folder: usize) -> bool {
    let mut cursor = source.parents[folder];
    while let Some(parent) = cursor {
        if parent == ancestor {
            return true;
        }
        cursor = source.parents[parent];
    }
    false
}

fn under(folder: &str, parent: &str) -> bool {
    folder != parent && std::path::Path::new(folder).starts_with(parent)
}

fn parent_of(folder: &str, folders: &[String]) -> Option<usize> {
    folders
        .iter()
        .enumerate()
        .filter(|(_, other)| under(folder, other))
        .max_by_key(|(_, other)| other.len())
        .map(|(index, _)| index)
}

fn summary(folders: &[String], states: &[ProfileState], workshop: ProfileState) -> String {
    let top = |wanted: ProfileState| {
        folders
            .iter()
            .zip(states)
            .filter(|(folder, state)| {
                **state == wanted
                    && parent_of(folder, folders).is_none_or(|parent| states[parent] != wanted)
            })
            .count()
            + usize::from(workshop == wanted)
    };
    let (hidden, excluded) = (top(ProfileState::Hide), top(ProfileState::Exclude));
    if hidden + excluded == 0 {
        return tr("settings-profiles-summary-none").to_owned();
    }
    tr_args!("settings-profiles-summary", hidden => hidden, excluded => excluded)
}
