#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTarget {
    Web,
    Android,
    AndroidDev,
    Ios,
    IosDev,
    Desktop,
}

impl RenderTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Android => "android",
            Self::AndroidDev => "android-dev",
            Self::Ios => "ios",
            Self::IosDev => "ios-dev",
            Self::Desktop => "desktop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropStage {
    Accepted,
    Lowered,
    Present,
    Consumed,
    Emitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRenderReport {
    pub route_path: String,
    pub accepted: Vec<ConsumedProp>,
    pub lowered: Vec<ConsumedProp>,
    pub present: Vec<ConsumedProp>,
    pub consumed: Vec<ConsumedProp>,
    pub emitted: Vec<ConsumedProp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub schema_version: u32,
    pub target: RenderTarget,
    pub routes: Vec<RouteRenderReport>,
    pub accepted: Vec<ConsumedProp>,
    pub lowered: Vec<ConsumedProp>,
    pub present: Vec<ConsumedProp>,
    pub consumed: Vec<ConsumedProp>,
    pub emitted: Vec<ConsumedProp>,
    pub consumed_props: Vec<ConsumedProp>,
}

impl RenderReport {
    pub fn new(target: RenderTarget, consumed: Vec<ConsumedProp>) -> Self {
        Self::from_stages(target, Vec::new(), Vec::new(), consumed.clone(), consumed.clone(), consumed)
    }

    pub fn from_routes(target: RenderTarget, routes: Vec<RouteRenderReport>) -> Self {
        let accepted = routes.iter().flat_map(|route| route.accepted.iter().cloned()).collect();
        let lowered = routes.iter().flat_map(|route| route.lowered.iter().cloned()).collect();
        let present = routes.iter().flat_map(|route| route.present.iter().cloned()).collect();
        let consumed = routes.iter().flat_map(|route| route.consumed.iter().cloned()).collect();
        let emitted = routes.iter().flat_map(|route| route.emitted.iter().cloned()).collect();
        Self::from_stages(target, routes, accepted, lowered, present, consumed)
            .with_emitted(emitted)
    }

    pub fn from_stages(
        target: RenderTarget,
        routes: Vec<RouteRenderReport>,
        accepted: Vec<ConsumedProp>,
        lowered: Vec<ConsumedProp>,
        present: Vec<ConsumedProp>,
        consumed: Vec<ConsumedProp>,
    ) -> Self {
        Self {
            schema_version: VIEW_IR_SCHEMA_VERSION,
            target,
            routes,
            accepted,
            lowered,
            present,
            consumed_props: consumed.clone(),
            consumed,
            emitted: Vec::new(),
        }
    }

    fn with_emitted(mut self, emitted: Vec<ConsumedProp>) -> Self {
        self.emitted = emitted;
        self
    }

    pub fn stage(&self, stage: PropStage) -> &[ConsumedProp] {
        match stage {
            PropStage::Accepted => &self.accepted,
            PropStage::Lowered => &self.lowered,
            PropStage::Present => &self.present,
            PropStage::Consumed => &self.consumed,
            PropStage::Emitted => &self.emitted,
        }
    }

    pub fn validate(&self) -> ComponentResult<()> {
        let mut registry = PropConsumptionRegistry::default();
        for entry in self.stage(PropStage::Consumed) {
            match entry.item {
                Some(item) => register_consumed_item(&mut registry, entry.component, item, entry.prop.clone(), entry.ir_field.clone()),
                None => register_consumed_prop(&mut registry, entry.component, entry.prop.clone(), entry.ir_field.clone()),
            }
        }
        registry.validate()
    }
}
