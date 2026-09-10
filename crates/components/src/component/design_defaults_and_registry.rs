#[derive(Default)]
struct ColorIdentifierRegistry {
    ids: HashMap<&'static str, u16>,
    names: Vec<&'static str>,
}

fn color_identifier_registry() -> &'static Mutex<ColorIdentifierRegistry> {
    static IDENTIFIERS: OnceLock<Mutex<ColorIdentifierRegistry>> = OnceLock::new();
    IDENTIFIERS.get_or_init(|| Mutex::new(ColorIdentifierRegistry::default()))
}

fn intern_color_identifier(value: &str) -> Option<u16> {
    let mut registry = color_identifier_registry()
        .lock()
        .expect("color identifier registry");
    if let Some(id) = registry.ids.get(value) {
        return Some(*id);
    }
    let id = u16::try_from(registry.names.len()).ok()?;
    let value = Box::leak(value.to_string().into_boxed_str());
    registry.ids.insert(value, id);
    registry.names.push(value);
    Some(id)
}

fn color_identifier(id: u16) -> &'static str {
    color_identifier_registry()
        .lock()
        .expect("color identifier registry")
        .names
        .get(usize::from(id))
        .copied()
        .expect("registered color identifier")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColorToken(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignConfig {
    pub default_theme: String,
    pub themes: Vec<DesignTheme>,
    pub defaults: DesignDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTheme {
    pub name: String,
    pub colors: BTreeMap<ColorToken, String>,
    pub radius: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignDefaults {
    pub font: BTreeMap<DesignComponentSlot, FontFamily>,
    pub radius: BTreeMap<DesignComponentSlot, RoundedSize>,
    pub shadow: BTreeMap<DesignComponentSlot, ShadowSize>,
    pub shadow_color: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub border: BTreeMap<DesignComponentSlot, BorderWidth>,
    pub border_color: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub scheme: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub variant: BTreeMap<DesignComponentSlot, ComponentVariant>,
    pub tabs_variant: BTreeMap<DesignComponentSlot, TabsVariant>,
    pub size: BTreeMap<DesignComponentSlot, ButtonSize>,
    pub label_floating: BTreeMap<DesignComponentSlot, bool>,
}

impl Default for DesignDefaults {
    fn default() -> Self {
        Self::with_builtin_defaults()
    }
}

impl DesignDefaults {
    pub fn empty() -> Self {
        Self {
            font: BTreeMap::new(),
            radius: BTreeMap::new(),
            shadow: BTreeMap::new(),
            shadow_color: BTreeMap::new(),
            border: BTreeMap::new(),
            border_color: BTreeMap::new(),
            scheme: BTreeMap::new(),
            variant: BTreeMap::new(),
            tabs_variant: BTreeMap::new(),
            size: BTreeMap::new(),
            label_floating: BTreeMap::new(),
        }
    }

    pub fn with_builtin_defaults() -> Self {
        let mut defaults = Self::empty();

        for (slot, scheme) in [
            (DesignComponentSlot::Button, ColorFamily::Primary),
            (DesignComponentSlot::IconButton, ColorFamily::Primary),
            (DesignComponentSlot::Card, ColorFamily::Surface),
            (DesignComponentSlot::Drawer, ColorFamily::Surface),
            (DesignComponentSlot::Toast, ColorFamily::Info),
            (DesignComponentSlot::Section, ColorFamily::Background),
            (DesignComponentSlot::Checkbox, ColorFamily::Primary),
            (DesignComponentSlot::Input, ColorFamily::Primary),
            (DesignComponentSlot::Date, ColorFamily::Primary),
            (DesignComponentSlot::Password, ColorFamily::Primary),
            (DesignComponentSlot::Select, ColorFamily::Primary),
            (DesignComponentSlot::Pin, ColorFamily::Primary),
            (DesignComponentSlot::SideNav, ColorFamily::Primary),
            (DesignComponentSlot::Sidebar, ColorFamily::Surface),
            (DesignComponentSlot::NavMenu, ColorFamily::Primary),
            (DesignComponentSlot::Chip, ColorFamily::Primary),
            (DesignComponentSlot::AppBar, ColorFamily::Surface),
            (DesignComponentSlot::Footer, ColorFamily::Surface),
            (DesignComponentSlot::Modal, ColorFamily::Surface),
            (DesignComponentSlot::Dropdown, ColorFamily::Surface),
            (DesignComponentSlot::Tooltip, ColorFamily::Surface),
            (DesignComponentSlot::Tabs, ColorFamily::Primary),
        ] {
            defaults.scheme.insert(slot, scheme);
        }

        for (slot, variant) in [
            (DesignComponentSlot::Button, ComponentVariant::Solid),
            (DesignComponentSlot::IconButton, ComponentVariant::Solid),
            (DesignComponentSlot::Card, ComponentVariant::Solid),
            (DesignComponentSlot::Drawer, ComponentVariant::Solid),
            (DesignComponentSlot::Toast, ComponentVariant::Solid),
            (DesignComponentSlot::Section, ComponentVariant::Solid),
            (DesignComponentSlot::Accordion, ComponentVariant::Ghost),
            (DesignComponentSlot::Input, ComponentVariant::Outlined),
            (DesignComponentSlot::Date, ComponentVariant::Outlined),
            (DesignComponentSlot::Password, ComponentVariant::Outlined),
            (DesignComponentSlot::Select, ComponentVariant::Outlined),
            (DesignComponentSlot::Pin, ComponentVariant::Outlined),
            (DesignComponentSlot::SideNav, ComponentVariant::Solid),
            (DesignComponentSlot::Sidebar, ComponentVariant::Solid),
            (DesignComponentSlot::NavMenu, ComponentVariant::Solid),
            (DesignComponentSlot::Chip, ComponentVariant::Solid),
            (DesignComponentSlot::AppBar, ComponentVariant::Solid),
            (DesignComponentSlot::Footer, ComponentVariant::Solid),
            (DesignComponentSlot::Modal, ComponentVariant::Solid),
            (DesignComponentSlot::Dropdown, ComponentVariant::Solid),
            (DesignComponentSlot::Tooltip, ComponentVariant::Solid),
        ] {
            defaults.variant.insert(slot, variant);
        }

        defaults
            .tabs_variant
            .insert(DesignComponentSlot::Tabs, TabsVariant::Pills);

        for slot in [
            DesignComponentSlot::Button,
            DesignComponentSlot::IconButton,
            DesignComponentSlot::Card,
            DesignComponentSlot::Toast,
        ] {
            defaults.radius.insert(slot, RoundedSize::Md);
        }

        defaults
    }

    pub fn with_builtin_overrides(configured: Self) -> Self {
        let mut defaults = Self::with_builtin_defaults();
        inherit_configured_ui(&mut defaults.radius, &configured.radius);
        inherit_configured_ui(&mut defaults.scheme, &configured.scheme);
        inherit_configured_ui(&mut defaults.variant, &configured.variant);
        inherit_configured_ui(&mut defaults.tabs_variant, &configured.tabs_variant);
        defaults.font.extend(configured.font);
        defaults.radius.extend(configured.radius);
        defaults.shadow.extend(configured.shadow);
        defaults.shadow_color.extend(configured.shadow_color);
        defaults.border.extend(configured.border);
        defaults.border_color.extend(configured.border_color);
        defaults.scheme.extend(configured.scheme);
        defaults.variant.extend(configured.variant);
        defaults.tabs_variant.extend(configured.tabs_variant);
        defaults.size.extend(configured.size);
        defaults.label_floating.extend(configured.label_floating);
        defaults
    }
}

fn inherit_configured_ui<T: Copy>(
    defaults: &mut BTreeMap<DesignComponentSlot, T>,
    configured: &BTreeMap<DesignComponentSlot, T>,
) {
    let Some(value) = configured.get(&DesignComponentSlot::Ui).copied() else {
        return;
    };
    for slot in DesignComponentSlot::all() {
        if defaults.contains_key(slot) && !configured.contains_key(slot) {
            defaults.insert(*slot, value);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DesignComponentSlot {
    Card,
    Button,
    IconButton,
    Drawer,
    Toast,
    Section,
    Accordion,
    Checkbox,
    Input,
    Date,
    DateRange,
    Color,
    Textarea,
    Password,
    Select,
    Pin,
    SideNav,
    Sidebar,
    NavMenu,
    AppBar,
    Footer,
    Modal,
    Dropdown,
    Tooltip,
    Tabs,
    Chip,
    Avatar,
    Text,
    Title,
    Ui,
}

