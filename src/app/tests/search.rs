use super::*;

#[test]
fn semantic_result_reorders_and_rolls() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("sunset");
    let b = wall("b.png", "static", 2, 0);
    seed(&mut app, &[a, b]);
    app.scene.seed_filter_cache(2);
    let _ =
        update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("warm sunset over water"))));
    let generation = app.tags.semantic.generation;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation,
        keys: vec![String::from("b.png"), String::from("a.png")],
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });
    assert_eq!(filtered_names(&app), ["b.png", "a.png"]);
    assert!(app.scene.filter_flip_running());
}

#[test]
fn semantic_result_picks_best_sandy() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    app.scene.mode = Mode::Sandy;
    app.scene.set_current(2, app.library_session.filtered.len());
    app.scene.filter_storm(Some(2));
    app.tags.semantic.search = String::from("seagulls");
    app.tags.semantic.generation = 4;

    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 4,
        keys: vec![String::from("b.png"), String::from("a.png")],
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });

    assert_eq!(filtered_names(&app), ["b.png", "a.png"]);
    assert_eq!(app.scene.current, 0);
    assert_eq!(app.scene.sandy_displayed(), 0);
}

#[test]
fn sort_reorders_semantic_results() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(
        &mut app,
        &[
            wall("new.png", "static", 1, 30),
            wall("old.png", "static", 2, 10),
            wall("excluded.png", "static", 3, 50),
        ],
    );
    app.tags.semantic.search = String::from("quiet landscape");
    app.tags.semantic.generation = 3;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 3,
        keys: vec![String::from("old.png"), String::from("new.png")],
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });
    assert_eq!(filtered_names(&app), ["old.png", "new.png"]);

    let _ = update(&mut app, Message::SetSort(String::from("date")));

    assert_eq!(filtered_names(&app), ["new.png", "old.png"]);
}

#[test]
fn semantic_result_filters() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    app.tags.semantic.search = String::from("sword -anime");
    app.tags.semantic.generation = 4;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 4,
        keys: vec![String::from("b.png")],
        exclusions: vec![String::from("anime")],
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });

    assert_eq!(filtered_names(&app), ["b.png"]);
    assert!(app.tags.semantic.resolved);
}

#[test]
fn empty_semantic_result_no_filler() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.tags.semantic.search = String::from("something impossible");
    app.tags.semantic.generation = 5;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 5,
        keys: Vec::new(),
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });

    assert!(filtered_names(&app).is_empty());
    assert!(app.tags.semantic.resolved);
}

#[test]
fn semantic_ranking_with_tag_facets() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("anime");
    let b = wall("b.png", "static", 2, 0);
    let mut c = wall("c.png", "static", 3, 0);
    c["tags"] = json!("anime");
    seed(&mut app, &[a, b, c]);
    app.tags.semantic.search = String::from("quiet room at night");
    app.tags.semantic.generation = 8;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 8,
        keys: vec![String::from("b.png"), String::from("c.png"), String::from("a.png")],
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("anime"), false)));
    assert_eq!(filtered_names(&app), ["c.png", "a.png"]);
    assert_eq!(app.tags.semantic.search, "quiet room at night");
}

#[test]
fn stale_semantic_response_ignored() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.tags.semantic.search = String::from("new query");
    app.tags.semantic.generation = 12;
    app.tags.semantic.pending = true;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: 11,
        keys: vec![String::from("b.png"), String::from("a.png")],
        exclusions: Vec::new(),
        query_ms: 9.0,
        search_ms: 0.8,
        error: None,
    });
    assert!(app.tags.semantic.pending);
    assert!(app.tags.semantic.ranked.is_empty());
    assert_eq!(filtered_names(&app), ["a.png", "b.png"]);
}

#[test]
fn filter_queue_collapses() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("sunset");
    let mut b = wall("b.png", "static", 2, 0);
    b["tags"] = json!("city");
    let c = wall("c.png", "static", 3, 0);
    seed(&mut app, &[a, b, c]);
    app.scene.seed_filter_cache(3);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("sunset"), false)));
    assert_eq!(filtered_names(&app), ["a.png"]);
    assert!(app.scene.filter_swap_active());
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("city"), false)));
    assert_eq!(filtered_names(&app), ["a.png"]);
    assert!(app.library_session.filter_transition_pending);
    let _ = update(&mut app, Message::ClearTags);
    assert!(app.library_session.filter_transition_pending);
    assert_eq!(filtered_names(&app), ["a.png"]);
    app.scene.finish_filter_swap();
    app.flush_pending_filter();
    assert!(!app.library_session.filter_transition_pending);
    assert_eq!(app.library_session.filtered.len(), 3);
}

#[test]
fn tag_query_exclusion() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("sunset,beach");
    let mut b = wall("b.png", "static", 2, 0);
    b["tags"] = json!("city");
    seed(&mut app, &[a, b]);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("sunset"), false)));
    assert_eq!(app.library_session.filters.tags, ["sunset"]);
    assert_eq!(filtered_names(&app), ["a.png"]);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("sunset"), true)));
    assert_eq!(app.library_session.filters.tags, ["-sunset"]);
    assert_eq!(filtered_names(&app), ["b.png"]);
    let _ = update(&mut app, Message::ClearTags);
    assert!(app.library_session.filters.tags.is_empty());
    assert_eq!(app.library_session.filtered.len(), 2);
}

#[test]
fn numeric_query_filters_indexed_catalog_metadata_live() {
    let mut hd = wall("hd.mp4", "video", 1, 0);
    hd["width"] = json!(1920);
    hd["height"] = json!(1080);
    hd["duration_ms"] = json!(20_000);
    hd["filesize"] = json!(4 * 1024 * 1024);
    let mut uhd = wall("uhd.mp4", "video", 2, 0);
    uhd["width"] = json!(3840);
    uhd["height"] = json!(2160);
    uhd["duration_ms"] = json!(120_000);
    uhd["filesize"] = json!(40 * 1024 * 1024);
    uhd["tags"] = json!("night");
    let mut app = test_app();
    seed(&mut app, &[hd, uhd]);

    let _ = update(
        &mut app,
        Message::Tag(TagMsg::QueryInput(String::from("res:>=2560x1440 duration:<3min"))),
    );

    assert_eq!(filtered_names(&app), ["uhd.mp4"]);
    assert!(app.library_session.filters.numeric.is_valid());
    assert_eq!(app.library_session.filters.numeric.chips().len(), 2);
    assert!(app.tags.matching_tags_open);

    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("night"), false)));
    assert_eq!(filtered_names(&app), ["uhd.mp4"]);
    assert!(app.tags.tag_search.contains("res:>=2560x1440"));
    assert!(!app.library_session.filters.numeric.is_empty());

    let _ = update(&mut app, Message::ClearTags);
    assert!(app.library_session.filters.numeric.is_empty());
    assert_eq!(app.library_session.filtered.len(), 2);
}

#[test]
fn malformed_numeric_query_is_visible_and_fail_closed() {
    let mut item = wall("wall.png", "static", 1, 0);
    item["width"] = json!(1920);
    item["height"] = json!(1080);
    let mut app = test_app();
    seed(&mut app, &[item]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("width:3840..1920"))));

    assert!(app.library_session.filtered.is_empty());
    let chips = app.library_session.filters.numeric.chips();
    assert_eq!(chips.len(), 1);
    assert!(chips[0].invalid);
}

#[test]
fn workshop_search_filters_library() {
    let mut scene = wall("Scene", "we", 1, 0);
    scene["key"] = json!("we:1234567890");
    let mut video = wall("Video", "video", 2, 0);
    video["we_id"] = json!("2345678901");
    let mut unrelated = wall("1234567890.mp4", "video", 3, 0);
    unrelated["tags"] = json!("2024");
    let mut app = test_app();
    seed(&mut app, &[scene, video, unrelated]);

    for (query, name) in [
        ("1234567890", "Scene"),
        ("https://steamcommunity.com/sharedfiles/filedetails/?id=2345678901", "Video"),
    ] {
        let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(query.into())));
        assert_eq!(filtered_names(&app), [name]);
        assert!(app.tags.matching_tags_open);
    }
    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput("9999999999".into())));
    assert!(app.library_session.filtered.is_empty());
    let _ = update(&mut app, Message::ClearTags);
    assert_eq!(app.library_session.filtered.len(), 3);
}

#[test]
fn workshop_search_bypasses_lens() {
    let mut item = wall("Scene", "we", 1, 0);
    item["we_id"] = json!("1234567890");
    let mut app = test_app();
    seed(&mut app, &[item]);
    app.tags.search_mode = SearchMode::Describe;
    let generation = app.tags.semantic.generation;

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput("1234567890".into())));

    assert_eq!(app.tags.search_mode, SearchMode::Describe);
    assert_eq!(filtered_names(&app), ["Scene"]);
    assert!(app.runtime_state.semantic.is_none());
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation,
        keys: Vec::new(),
        exclusions: Vec::new(),
        query_ms: 0.0,
        search_ms: 0.0,
        error: None,
    });
    assert_eq!(filtered_names(&app), ["Scene"]);
}

#[test]
fn describe_accepts_workshop_syntax() {
    let mut item = wall("Scene", "we", 1, 0);
    item["we_id"] = json!("1234567890");
    let mut app = test_app();
    seed(&mut app, &[item]);
    app.tags.search_mode = SearchMode::Describe;
    for text in [
        "id:1234567890",
        "workshop:1234567890",
        "1234567890",
        "https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890",
        "steam://url/CommunityFilePage/1234567890",
    ] {
        let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(text.into())));
        assert_eq!(filtered_names(&app), ["Scene"]);
        assert_eq!(app.tags.search_mode, SearchMode::Describe);
        assert!(app.tags.semantic.resolved);
    }
    for text in ["3", "3d", "3d abstract"] {
        let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(text.into())));
        assert_eq!(app.tags.search_mode, SearchMode::Describe);
        assert_eq!(app.tags.semantic.search, text);
    }
    assert!(!app.tags.semantic.resolved);
}

#[test]
fn free_form_query_stays_semantic() {
    let mut matching = wall("matching.png", "static", 1, 0);
    matching["tags"] = json!("woman,art,animal,ocean");
    let mut with_people = wall("with-people.png", "static", 2, 0);
    with_people["tags"] = json!("woman,art,animal,ocean,people");
    let mut unrelated = wall("unrelated.png", "static", 3, 0);
    unrelated["tags"] = json!("city,night");
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    seed(&mut app, &[matching, with_people, unrelated]);

    let _ = update(
        &mut app,
        Message::Tag(TagMsg::QueryInput(String::from(
            "women drawing animals by the sea with no people",
        ))),
    );

    assert!(app.library_session.filters.tags.is_empty());
    assert_eq!(app.library_session.filtered.len(), 3);
    let generation = app.tags.semantic.generation;
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation,
        keys: vec![
            String::from("matching.png"),
            String::from("with-people.png"),
            String::from("unrelated.png"),
        ],
        exclusions: vec![String::from("people")],
        query_ms: 7.5,
        search_ms: 0.5,
        error: None,
    });
    assert_eq!(filtered_names(&app), ["matching.png", "unrelated.png"]);
}

#[test]
fn tag_search_controls_set_state() {
    let mut app = test_app();
    let _ = update(&mut app, Message::Tag(TagMsg::MatchMode(true)));
    let _ = update(&mut app, Message::Tag(TagMsg::SortAz(true)));
    let _ = update(&mut app, Message::Tag(TagMsg::ToggleMatchingTags));
    assert!(app.library_session.filters.tags_match_any);
    assert!(app.tags.sort_az);
    assert!(app.tags.matching_tags_open);

    let _ = update(&mut app, Message::Tag(TagMsg::MatchMode(false)));
    let _ = update(&mut app, Message::Tag(TagMsg::SortAz(false)));
    let _ = update(&mut app, Message::Tag(TagMsg::ToggleMatchingTags));
    assert!(!app.library_session.filters.tags_match_any);
    assert!(!app.tags.sort_az);
    assert!(!app.tags.matching_tags_open);
}

#[test]
fn tag_search_skips_semantic() {
    let mut app = test_app();
    let mut sunset = wall("sunset.png", "static", 1, 0);
    sunset["tags"] = json!("sunset,orange");
    let mut forest = wall("forest.png", "static", 2, 0);
    forest["tags"] = json!("forest,green");
    seed(&mut app, &[sunset, forest]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("sun"))));

    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    assert_eq!(app.tags.tag_search, "sun");
    assert!(app.tags.semantic.search.is_empty());
    assert!(app.runtime_state.semantic.is_none());
    assert_eq!(
        app.tag_cloud_entries().iter().map(|entry| entry.tag.as_str()).collect::<Vec<_>>(),
        ["sunset"]
    );
}

#[test]
fn space_commits_backspace_removes_tokens() {
    let mut app = test_app();
    let mut beach = wall("beach.png", "static", 1, 0);
    beach["tags"] = json!("seagull,ocean");
    let mut lake = wall("lake.png", "static", 2, 0);
    lake["tags"] = json!("ocean");
    seed(&mut app, &[beach, lake]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagull"))));
    assert!(app.library_session.filters.tags.is_empty());

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagull "))));
    assert_eq!(app.library_session.filters.tags, ["seagull"]);
    assert!(app.tags.matching_tags_open);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagull oce"))));
    assert_eq!(app.library_session.filters.tags, ["seagull"]);
    assert!(app.tags.matching_tags_open);
    assert_eq!(
        app.tag_cloud_entries()
            .iter()
            .filter(|entry| !entry.selected && !entry.excluded)
            .map(|entry| entry.tag.as_str())
            .collect::<Vec<_>>(),
        ["ocean"]
    );

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagull ocean "))));
    assert_eq!(app.library_session.filters.tags, ["seagull", "ocean"]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagull ocea"))));
    assert_eq!(app.library_session.filters.tags, ["seagull"]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("seagul"))));
    assert!(app.library_session.filters.tags.is_empty());
}

#[test]
fn extra_space_keeps_combinations_visible() {
    let mut app = test_app();
    let mut mountain_ocean = wall("coast.png", "static", 1, 0);
    mountain_ocean["tags"] = json!("mountain,ocean");
    let mut mountain_forest = wall("forest.png", "static", 2, 0);
    mountain_forest["tags"] = json!("mountain,forest");
    seed(&mut app, &[mountain_ocean, mountain_forest]);

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain "))));
    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain  "))));
    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain "))));

    assert_eq!(app.library_session.filters.tags, ["mountain"]);
    assert!(app.tags.matching_tags_open);
    let entries = app.tag_cloud_entries();
    let combinations: Vec<&str> = entries
        .iter()
        .filter(|entry| !entry.selected && !entry.excluded)
        .map(|entry| entry.tag.as_str())
        .collect();
    assert_eq!(combinations, ["forest", "ocean"]);
}

#[test]
fn tab_completion_preserves_other_tags() {
    let mut app = test_app();
    let mut landscape = wall("landscape.png", "static", 1, 0);
    landscape["tags"] = json!("mountain,ocean,forest");
    seed(&mut app, &[landscape]);
    app.tags.cloud_open = true;

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain "))));
    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain oce"))));
    let _ = update(&mut app, Message::Tag(TagMsg::Autocomplete));
    assert_eq!(app.library_session.filters.tags, ["mountain", "ocean"]);
    assert_eq!(app.tags.tag_search, "mountain ocean ");

    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput(String::from("mountain ocean for"))));
    let _ = update(&mut app, Message::Tag(TagMsg::Autocomplete));
    assert_eq!(app.library_session.filters.tags, ["mountain", "ocean", "forest"]);

    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("ocean"), false)));
    assert_eq!(app.library_session.filters.tags, ["mountain", "forest"]);
    assert_eq!(app.tags.tag_search, "mountain forest ");
}

#[test]
fn tag_click_mirrors_search_field() {
    let mut app = test_app();
    let mut beach = wall("beach.png", "static", 1, 0);
    beach["tags"] = json!("seagull,ocean");
    seed(&mut app, &[beach]);

    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("seagull"), false)));

    assert_eq!(app.library_session.filters.tags, ["seagull"]);
    assert_eq!(app.tags.tag_search, "seagull ");
}

#[test]
fn search_modes_keep_separate_text() {
    let mut app = test_app();
    app.tags.tag_search = String::from("forest");

    let _ = update(&mut app, Message::Tag(TagMsg::SearchMode(SearchMode::Describe)));
    assert_eq!(app.tags.search_mode, SearchMode::Describe);
    assert_eq!(app.tags.tag_search, "forest");
    app.tags.semantic.search = String::from("quiet lake at night");

    let _ = update(&mut app, Message::Tag(TagMsg::SearchMode(SearchMode::Tags)));
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    assert_eq!(app.tags.semantic.search, "quiet lake at night");
    assert!(app.runtime_state.semantic.is_none());
}

#[test]
fn settings_hide_tag_search() {
    let mut app = test_app();
    app.tags.cloud_open = true;
    app.tags.tag_search = String::from("sunset");
    assert!(super::view::tag_cloud_visible(app.tags.cloud_open, app.panels.settings.open, 1080.0));

    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.tags.cloud_open);
    assert_eq!(app.tags.tag_search, "sunset");
    assert!(!super::view::tag_cloud_visible(app.tags.cloud_open, app.panels.settings.open, 1080.0));

    let _ = update(&mut app, Message::ToggleSettings);
    assert!(super::view::tag_cloud_visible(app.tags.cloud_open, app.panels.settings.open, 1080.0));
    assert_eq!(app.tags.tag_search, "sunset");
}

#[test]
fn warmup_generation_dropped() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.tags.search_mode = SearchMode::Describe;
    let before = app.library_session.filtered.clone();

    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation: u64::MAX,
        keys: vec![String::from("a.png")],
        exclusions: Vec::new(),
        query_ms: 8.0,
        search_ms: 0.7,
        error: None,
    });

    assert!(app.tags.semantic.ranked.is_empty());
    assert!(!app.tags.semantic.resolved);
    assert_eq!(app.library_session.filtered, before);
}

#[test]
fn close_tag_cloud_routes_overlay() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(app.tags.cloud_open);
    app.tags.matching_tags_open = true;
    app.tags.semantic.search = String::from("sunset");
    app.tags.tag_search = String::from("sun");
    let _ = update(&mut app, Message::CloseTagCloud);
    assert!(!app.tags.cloud_open);
    assert!(!app.tags.matching_tags_open);
    assert!(app.tags.semantic.search.is_empty());
    assert!(app.tags.tag_search.is_empty());
    assert!(app.tags.semantic.ranked.is_empty());
    let dispatcher = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/update/dispatcher.rs"),
    )
    .expect("read dispatcher.rs");
    assert!(!dispatcher.contains("cloud_open = false"));
}
