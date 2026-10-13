mod numeric;
#[allow(clippy::module_inception)]
mod search;
mod tests;

pub use numeric::WORKSHOP_KEY;
pub use numeric::parse_numeric_query;
pub use numeric::{NumericQuery, QueryChip};
pub use search::{
    LibraryQuery, TagEntry, parse_library_search, recompute_tag_cloud, semantic_query,
    suggest_completion,
};
