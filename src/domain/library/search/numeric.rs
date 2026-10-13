use crate::domain::library::catalog::Wallpaper;

pub const WORKSHOP_KEY: &str = "workshop";
const WORKSHOP_ALIAS: &str = "id";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericField {
    WorkshopId,
    Width,
    Height,
    Resolution,
    Duration,
    FileSize,
}

impl NumericField {
    fn parse(value: &str) -> Option<Self> {
        match value {
            WORKSHOP_KEY | WORKSHOP_ALIAS => Some(Self::WorkshopId),
            "width" | "w" => Some(Self::Width),
            "height" | "h" => Some(Self::Height),
            "resolution" | "res" | "dimensions" | "dims" => Some(Self::Resolution),
            "duration" | "dur" | "length" => Some(Self::Duration),
            "size" | "filesize" | "file-size" | "bytes" => Some(Self::FileSize),
            _ => None,
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::WorkshopId => WORKSHOP_KEY,
            Self::Width => "width",
            Self::Height => "height",
            Self::Resolution => "res",
            Self::Duration => "duration",
            Self::FileSize => "size",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumericBound {
    pub value: u64,
    pub inclusive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumericRange {
    pub lower: Option<NumericBound>,
    pub upper: Option<NumericBound>,
}

impl NumericRange {
    pub const fn exact(value: u64) -> Self {
        let bound = NumericBound { value, inclusive: true };
        Self { lower: Some(bound), upper: Some(bound) }
    }

    fn matches(self, value: i64) -> bool {
        let Ok(value) = u64::try_from(value) else { return false };
        if value == 0 {
            return false;
        }
        let above = self.lower.is_none_or(|bound| {
            if bound.inclusive { value >= bound.value } else { value > bound.value }
        });
        let below = self.upper.is_none_or(|bound| {
            if bound.inclusive { value <= bound.value } else { value < bound.value }
        });
        above && below
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumericConstraint {
    WorkshopId(u64),
    Scalar(NumericRange),
    Resolution { width: NumericRange, height: NumericRange },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumericPredicate {
    pub field: NumericField,
    pub constraint: NumericConstraint,
    pub excluded: bool,
    label: String,
}

impl NumericPredicate {
    fn matches(&self, item: &Wallpaper) -> bool {
        let matched = match (&self.constraint, self.field) {
            (NumericConstraint::WorkshopId(id), NumericField::WorkshopId) => {
                item.engine_id().parse::<u64>() == Ok(*id)
            }
            (NumericConstraint::Scalar(range), NumericField::Width) => range.matches(item.width),
            (NumericConstraint::Scalar(range), NumericField::Height) => range.matches(item.height),
            (NumericConstraint::Scalar(range), NumericField::Duration) => {
                range.matches(item.duration_ms)
            }
            (NumericConstraint::Scalar(range), NumericField::FileSize) => {
                range.matches(item.filesize)
            }
            (NumericConstraint::Resolution { width, height }, NumericField::Resolution) => {
                width.matches(item.width) && height.matches(item.height)
            }
            _ => false,
        };
        matched != self.excluded
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryChip {
    pub label: String,
    pub invalid: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NumericQuery {
    groups: Vec<Vec<NumericPredicate>>,
    invalid: Vec<String>,
}

impl NumericQuery {
    pub fn has_workshop(&self) -> bool {
        self.groups.iter().flatten().any(|term| term.field == NumericField::WorkshopId)
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.invalid.is_empty()
    }

    #[cfg(test)]
    pub fn is_valid(&self) -> bool {
        self.invalid.is_empty()
    }

    #[cfg(test)]
    pub fn groups(&self) -> &[Vec<NumericPredicate>] {
        &self.groups
    }

    #[cfg(test)]
    pub fn invalid(&self) -> &[String] {
        &self.invalid
    }

    pub fn matches(&self, item: &Wallpaper) -> bool {
        self.invalid.is_empty()
            && self.groups.iter().all(|group| group.iter().any(|term| term.matches(item)))
    }

    pub fn chips(&self) -> Vec<QueryChip> {
        let mut chips: Vec<QueryChip> = self
            .groups
            .iter()
            .map(|group| QueryChip {
                label: group.iter().map(NumericPredicate::label).collect::<Vec<_>>().join(" OR "),
                invalid: false,
            })
            .collect();
        chips.extend(
            self.invalid.iter().map(|label| QueryChip { label: label.clone(), invalid: true }),
        );
        chips
    }

    pub fn query_text(&self) -> String {
        self.groups
            .iter()
            .map(|group| group.iter().map(NumericPredicate::label).collect::<Vec<_>>().join(" | "))
            .chain(self.invalid.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn parse_numeric_query(text: &str) -> NumericQuery {
    let tokens = numeric_tokens(&text.to_lowercase());
    let mut query = NumericQuery::default();
    let mut previous_numeric = None;
    for (index, token) in tokens.iter().enumerate() {
        let Some(parsed) = parse_predicate(token) else { continue };
        let alternative = previous_numeric.is_some_and(|previous| {
            let connectors = &tokens[previous + 1..index];
            !connectors.is_empty()
                && connectors.iter().all(|connector| connector == "|" || connector == "or")
        });
        match parsed {
            Ok(predicate) => {
                if alternative && let Some(group) = query.groups.last_mut() {
                    group.push(predicate);
                } else {
                    query.groups.push(vec![predicate]);
                }
            }
            Err(()) => query.invalid.push(token.clone()),
        }
        previous_numeric = Some(index);
    }
    query
}

pub fn strip_numeric_query(text: &str, tags: &[&str]) -> String {
    let is_numeric = |token: &str| !tags.contains(&token) && is_numeric_token(token);
    let has_numeric = text
        .split_whitespace()
        .flat_map(|token| token.trim_matches('|').split('|'))
        .any(is_numeric);
    if !has_numeric {
        return text.to_string();
    }
    text.split_whitespace()
        .filter(|token| {
            let bare = token.trim_matches('|');
            if bare.eq_ignore_ascii_case("and") || bare.eq_ignore_ascii_case("or") {
                return false;
            }
            !bare.split('|').any(is_numeric)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn workshop_id(token: &str) -> Option<u64> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    parse_pixels(token).ok()
}

fn is_numeric_token(token: &str) -> bool {
    let bare = token.trim_start_matches('-');
    workshop_id(bare).is_some()
        || bare.split_once(':').and_then(|(field, _)| NumericField::parse(field)).is_some()
}

fn numeric_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_whitespace() || character == '|' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if character == '|' {
                tokens.push(String::from("|"));
            }
        } else {
            current.push(character);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_predicate(token: &str) -> Option<Result<NumericPredicate, ()>> {
    let (excluded, bare) = token.strip_prefix('-').map_or((false, token), |value| (true, value));
    let (field, expression) = if workshop_id(bare).is_some() {
        (NumericField::WorkshopId, bare)
    } else {
        let (field, expression) = bare.split_once(':')?;
        (NumericField::parse(field)?, expression)
    };
    let parsed = parse_constraint(field, expression).map(|constraint| NumericPredicate {
        field,
        excluded,
        label: format!(
            "{}{key}:{}",
            if excluded { "-" } else { "" },
            if let NumericConstraint::WorkshopId(id) = constraint {
                id.to_string()
            } else {
                expression.to_string()
            },
            key = field.key()
        ),
        constraint,
    });
    Some(parsed)
}

fn parse_constraint(field: NumericField, expression: &str) -> Result<NumericConstraint, ()> {
    if field == NumericField::WorkshopId {
        return workshop_id(expression).map(NumericConstraint::WorkshopId).ok_or(());
    }
    if expression.is_empty() {
        return Err(());
    }
    if field == NumericField::Resolution {
        parse_resolution_constraint(expression)
    } else {
        parse_scalar_constraint(field, expression).map(NumericConstraint::Scalar)
    }
}

fn parse_scalar_constraint(field: NumericField, expression: &str) -> Result<NumericRange, ()> {
    if let Some(value) = expression.strip_prefix(">=") {
        return Ok(lower_range(parse_measure(field, value)?, true));
    }
    if let Some(value) = expression.strip_prefix('>') {
        return Ok(lower_range(parse_measure(field, value)?, false));
    }
    if let Some(value) = expression.strip_prefix("<=") {
        return Ok(upper_range(parse_measure(field, value)?, true));
    }
    if let Some(value) = expression.strip_prefix('<') {
        return Ok(upper_range(parse_measure(field, value)?, false));
    }
    let expression = expression.strip_prefix('=').unwrap_or(expression);
    if let Some((lower, upper)) = expression.split_once("..") {
        let lower = (!lower.is_empty())
            .then(|| {
                parse_measure(field, lower).map(|value| NumericBound { value, inclusive: true })
            })
            .transpose()?;
        let upper = (!upper.is_empty())
            .then(|| {
                parse_measure(field, upper).map(|value| NumericBound { value, inclusive: true })
            })
            .transpose()?;
        if lower.is_none() && upper.is_none() {
            return Err(());
        }
        if lower.zip(upper).is_some_and(|(lower, upper)| lower.value > upper.value) {
            return Err(());
        }
        return Ok(NumericRange { lower, upper });
    }
    Ok(NumericRange::exact(parse_measure(field, expression)?))
}

fn parse_resolution_constraint(expression: &str) -> Result<NumericConstraint, ()> {
    let (operator, expression) = if let Some(value) = expression.strip_prefix(">=") {
        (Some((true, true)), value)
    } else if let Some(value) = expression.strip_prefix('>') {
        (Some((true, false)), value)
    } else if let Some(value) = expression.strip_prefix("<=") {
        (Some((false, true)), value)
    } else if let Some(value) = expression.strip_prefix('<') {
        (Some((false, false)), value)
    } else {
        (None, expression.strip_prefix('=').unwrap_or(expression))
    };
    if let Some((lower, inclusive)) = operator {
        let (width, height) = parse_resolution_value(expression)?;
        let width =
            if lower { lower_range(width, inclusive) } else { upper_range(width, inclusive) };
        let height =
            if lower { lower_range(height, inclusive) } else { upper_range(height, inclusive) };
        return Ok(NumericConstraint::Resolution { width, height });
    }
    if let Some((lower, upper)) = expression.split_once("..") {
        if lower.is_empty() && upper.is_empty() {
            return Err(());
        }
        let from = (!lower.is_empty()).then(|| parse_resolution_value(lower)).transpose()?;
        let to = (!upper.is_empty()).then(|| parse_resolution_value(upper)).transpose()?;
        if from.zip(to).is_some_and(|(from, to)| from.0 > to.0 || from.1 > to.1) {
            return Err(());
        }
        let width = NumericRange {
            lower: from.map(|value| NumericBound { value: value.0, inclusive: true }),
            upper: to.map(|value| NumericBound { value: value.0, inclusive: true }),
        };
        let height = NumericRange {
            lower: from.map(|value| NumericBound { value: value.1, inclusive: true }),
            upper: to.map(|value| NumericBound { value: value.1, inclusive: true }),
        };
        return Ok(NumericConstraint::Resolution { width, height });
    }
    let (width, height) = parse_resolution_value(expression)?;
    Ok(NumericConstraint::Resolution {
        width: NumericRange::exact(width),
        height: NumericRange::exact(height),
    })
}

fn lower_range(value: u64, inclusive: bool) -> NumericRange {
    NumericRange { lower: Some(NumericBound { value, inclusive }), upper: None }
}

fn upper_range(value: u64, inclusive: bool) -> NumericRange {
    NumericRange { lower: None, upper: Some(NumericBound { value, inclusive }) }
}

fn parse_resolution_value(value: &str) -> Result<(u64, u64), ()> {
    let (width, height) = value.split_once(['x', '×']).ok_or(())?;
    Ok((parse_pixels(width)?, parse_pixels(height)?))
}

fn parse_measure(field: NumericField, value: &str) -> Result<u64, ()> {
    match field {
        NumericField::Width | NumericField::Height => parse_pixels(value),
        NumericField::Duration => parse_scaled(value, duration_unit),
        NumericField::FileSize => parse_scaled(value, size_unit),
        NumericField::Resolution | NumericField::WorkshopId => Err(()),
    }
}

fn parse_pixels(value: &str) -> Result<u64, ()> {
    let number = value.strip_suffix("px").unwrap_or(value);
    let parsed = number.parse::<u64>().map_err(|_| ())?;
    (parsed > 0).then_some(parsed).ok_or(())
}

fn parse_scaled(value: &str, unit: fn(&str) -> (&str, f64)) -> Result<u64, ()> {
    let (number, scale) = unit(value);
    let number = number.parse::<f64>().map_err(|_| ())?;
    let scaled = number * scale;
    if !scaled.is_finite() || scaled <= 0.0 || scaled > u64::MAX as f64 {
        return Err(());
    }
    Ok(scaled.round() as u64)
}

fn duration_unit(value: &str) -> (&str, f64) {
    for (suffix, scale) in
        [("ms", 1.0), ("s", 1_000.0), ("min", 60_000.0), ("m", 60_000.0), ("h", 3_600_000.0)]
    {
        if let Some(number) = value.strip_suffix(suffix) {
            return (number, scale);
        }
    }
    (value, 1_000.0)
}

fn size_unit(value: &str) -> (&str, f64) {
    for (suffix, scale) in [
        ("tib", 1024_f64.powi(4)),
        ("gib", 1024_f64.powi(3)),
        ("mib", 1024_f64.powi(2)),
        ("kib", 1024.0),
        ("tb", 1000_f64.powi(4)),
        ("gb", 1000_f64.powi(3)),
        ("mb", 1000_f64.powi(2)),
        ("kb", 1000.0),
        ("b", 1.0),
    ] {
        if let Some(number) = value.strip_suffix(suffix) {
            return (number, scale);
        }
    }
    (value, 1.0)
}
