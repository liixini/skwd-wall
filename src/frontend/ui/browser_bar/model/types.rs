#[derive(Clone)]
pub enum BrowserAct {
    Close,
    Label,
    MaxDuration(&'static str),
    CatalogFilter(&'static str, String),
}
