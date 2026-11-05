use crate::i18n::tr;

pub(crate) fn sort_label_key(key: &str) -> &'static str {
    match key {
        "0" => "browser-top",
        "1" => "browser-new",
        "21" => "browser-updated",
        "9" => "browser-subscribers",
        _ => "browser-trend",
    }
}

pub(crate) fn trend_days_label_key(key: &str) -> &'static str {
    match key {
        "1" => "browser-1-day",
        "3" => "browser-3-days",
        _ => "browser-7-days",
    }
}

pub(crate) fn type_label_key(key: &str) -> &'static str {
    match key {
        "Scene" => "browser-scene",
        "Video" => "browser-video",
        _ => "browser-scene-video",
    }
}

pub(crate) fn resolution_label(key: &str) -> &'static str {
    match key {
        "1920 x 1080" => "1080p",
        "2560 x 1440" => "2K",
        "3840 x 2160" => "4K",
        "2560 x 1080" => "UW",
        "3440 x 1440" => "UWQHD",
        "3840 x 1080" => tr("browser-dual"),
        _ => tr("browser-any"),
    }
}

pub(crate) fn category_label_key(key: &str) -> &'static str {
    match key {
        "Abstract" => "browser-abstract",
        "Animal" => "browser-animal",
        "Anime" => "browser-anime",
        "CGI" => "browser-cgi",
        "Cyberpunk" => "browser-cyberpunk",
        "Fantasy" => "browser-fantasy",
        "Game" => "browser-game",
        "Girls" => "browser-girls",
        "Guys" => "browser-guys",
        "Landscape" => "browser-landscape",
        "Medieval" => "browser-medieval",
        "Music" => "browser-music",
        "Nature" => "browser-nature",
        "Pixel art" => "browser-pixel-art",
        "Relaxing" => "browser-relaxing",
        "Retro" => "browser-retro",
        "Sci-Fi" => "browser-sci-fi",
        "Technology" => "browser-technology",
        "Vehicle" => "browser-vehicle",
        _ => "browser-all",
    }
}
