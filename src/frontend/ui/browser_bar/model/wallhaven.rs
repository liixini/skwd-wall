pub(crate) fn sort_label_key(key: &str) -> &'static str {
    match key {
        "toplist" => "browser-top",
        "hot" => "browser-hot",
        "date_added" => "browser-new",
        "relevance" => "browser-relevant",
        "views" => "browser-views",
        "favorites" => "browser-favourites",
        _ => "browser-random",
    }
}

pub(crate) fn range_label_key(key: &str) -> &'static str {
    match key {
        "1d" => "browser-day",
        "3d" => "browser-3-days",
        "1w" => "browser-week",
        "1M" => "browser-month",
        "3M" => "browser-3-months",
        "6M" => "browser-6-months",
        _ => "browser-year",
    }
}
