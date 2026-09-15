#[derive(Debug, Clone, PartialEq)]
pub struct GeometryEnvironment {
    pub width: f64,
    pub height: f64,
    pub text_scale: f64,
    pub locale: String,
    pub theme_id: String,
    pub content_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryMeasurement {
    pub node_id: String,
    pub parent_id: String,
    pub rect: GeometryRect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometrySnapshot {
    pub target: RenderTarget,
    pub case_id: String,
    pub environment: GeometryEnvironment,
    pub density: f64,
    pub regions: Vec<GeometryMeasurement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryDifference {
    pub node_id: String,
    pub expected: Option<GeometryRect>,
    pub actual: Option<GeometryRect>,
    pub composition_changed: bool,
}

impl GeometryRect {
    fn logical(self, density: f64) -> Self {
        Self {
            x: self.x / density,
            y: self.y / density,
            width: self.width / density,
            height: self.height / density,
        }
    }

    fn edges(self) -> [f64; 4] {
        [self.x, self.y, self.x + self.width, self.y + self.height]
    }

    fn valid(self) -> bool {
        self.edges().iter().all(|value| value.is_finite())
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width >= 0.0
            && self.height >= 0.0
    }
}

impl GeometrySnapshot {
    pub fn validate(&self) -> Result<(), String> {
        if !self.density.is_finite() || self.density <= 0.0 {
            return Err("Geometry density must be finite and positive".into());
        }
        let environment = &self.environment;
        if [
            environment.width,
            environment.height,
            environment.text_scale,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
            || environment.locale.is_empty()
            || environment.theme_id.is_empty()
            || environment.content_id.is_empty()
            || self.case_id.is_empty()
        {
            return Err("Geometry environment must identify a valid reproducible case".into());
        }
        if self.regions.is_empty() {
            return Err("Geometry evidence must contain measured regions".into());
        }
        let mut ids = BTreeSet::new();
        for region in &self.regions {
            if region.node_id.is_empty() || !ids.insert(&region.node_id) {
                return Err("Geometry region identities must be nonempty and unique".into());
            }
            if region.node_id == region.parent_id
                || !region.rect.valid()
                || !region.rect.logical(self.density).valid()
            {
                return Err(format!("Invalid geometry for {}", region.node_id));
            }
        }
        Ok(())
    }
}

pub fn compare_geometry(
    reference: &GeometrySnapshot,
    actual: &GeometrySnapshot,
    tolerance: f64,
) -> Result<Vec<GeometryDifference>, String> {
    reference.validate()?;
    actual.validate()?;
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err("Geometry tolerance must be finite and nonnegative".into());
    }
    if reference.case_id != actual.case_id || reference.environment != actual.environment {
        return Err("Geometry comparison requires the same case and logical environment".into());
    }
    let expected: std::collections::BTreeMap<_, _> = reference
        .regions
        .iter()
        .map(|region| (region.node_id.as_str(), region))
        .collect();
    let measured: std::collections::BTreeMap<_, _> = actual
        .regions
        .iter()
        .map(|region| (region.node_id.as_str(), region))
        .collect();
    let ids: BTreeSet<_> = expected.keys().chain(measured.keys()).copied().collect();
    let mut differences = Vec::new();
    for id in ids {
        let left = expected.get(id);
        let right = measured.get(id);
        let expected_rect = left.map(|region| region.rect.logical(reference.density));
        let actual_rect = right.map(|region| region.rect.logical(actual.density));
        let composition_changed =
            left.map(|region| &region.parent_id) != right.map(|region| &region.parent_id);
        let geometry_changed = match (expected_rect, actual_rect) {
            (Some(left), Some(right)) => left
                .edges()
                .iter()
                .zip(right.edges())
                .any(|(left, right)| (left - right).abs() > tolerance),
            _ => true,
        };
        if composition_changed || geometry_changed {
            differences.push(GeometryDifference {
                node_id: id.into(),
                expected: expected_rect,
                actual: actual_rect,
                composition_changed,
            });
        }
    }
    Ok(differences)
}
