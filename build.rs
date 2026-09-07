use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    generate_taxonomy_aliases();
    generate_embedded_locales();
}

fn generate_taxonomy_aliases() {
    const TAXONOMY_PATH: &str = "data/contracts/lens-taxonomy-v1.json";
    println!("cargo::rerun-if-changed={TAXONOMY_PATH}");

    let source = std::fs::read_to_string(TAXONOMY_PATH).expect("read wallpaper taxonomy");
    let taxonomy: serde_json::Value =
        serde_json::from_str(&source).expect("parse wallpaper taxonomy");
    assert_eq!(taxonomy["format"], 1);
    assert_eq!(taxonomy["id"], "skwd-wallpaper");
    assert_eq!(taxonomy["version"], "taxonomy-v1");
    let categories = taxonomy["categories"].as_object().expect("taxonomy categories");
    let mut aliases = BTreeMap::new();
    for concepts in categories.values() {
        for concept in concepts.as_array().expect("taxonomy category concepts") {
            let tag = concept["tag"].as_str().expect("taxonomy concept tag");
            let Some(concept_aliases) = concept["aliases"].as_array() else {
                continue;
            };
            for alias in concept_aliases {
                let normalized =
                    alias.as_str().expect("taxonomy alias").to_lowercase().replace([' ', '_'], "-");
                if let Some(existing) = aliases.insert(normalized.clone(), tag.to_string()) {
                    assert_eq!(existing, tag, "alias {normalized:?}");
                }
            }
        }
    }

    let mut generated = String::from("const TAXONOMY_ALIASES: &[(&str, &str)] = &[\n");
    for (alias, tag) in aliases {
        writeln!(generated, "    ({alias:?}, {tag:?}),").expect("write generated alias");
    }
    generated.push_str("];\n");

    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("wallpaper_taxonomy_aliases.rs");
    std::fs::write(output, generated).expect("write wallpaper taxonomy aliases");
}

fn generate_embedded_locales() {
    println!("cargo:rerun-if-changed=locales");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must provide OUT_DIR"));
    let generated = out_dir.join("embedded_locales.rs");
    let locales = Path::new("locales");

    let en_us = fluent_files(&locales.join("en-US")).expect("failed to scan en-US Fluent files");
    let sv_se = fluent_files(&locales.join("sv-SE")).expect("failed to scan sv-SE Fluent files");
    let es_es = fluent_files(&locales.join("es-ES")).expect("failed to scan es-ES Fluent files");

    let source = format!(
        "pub const EN_US_RESOURCES: &[&str] = &{};\n\
         pub const SV_SE_RESOURCES: &[&str] = &{};\n\
         pub const ES_ES_RESOURCES: &[&str] = &{};\n",
        include_array(&en_us),
        include_array(&sv_se),
        include_array(&es_es),
    );
    fs::write(generated, source).expect("failed to generate embedded Fluent resource list");
}

fn fluent_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        if !directory.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_dir() {
                visit(&path, files)?;
            } else if file_type.is_file()
                && path.extension().is_some_and(|extension| extension == "ftl")
            {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn include_array(files: &[PathBuf]) -> String {
    let entries = files
        .iter()
        .map(|path| {
            let relative = path.to_string_lossy().replace('\\', "/");
            format!("include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{relative}\"))")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{entries}]")
}
