use super::{
    component_prop_contract, register_consumed_prop, view_prop_declared, PropConsumptionRegistry,
    ViewPropDefinition,
    ViewItemKind, ViewPropOwner, VIEW_IR_SCHEMA_VERSION, VIEW_PROP_INVENTORY,
};

#[test]
fn consumption_registry_deduplicates_declarative_registrations() {
    let mut registry = PropConsumptionRegistry::default();
    register_consumed_prop(
        &mut registry,
        BuiltinComponent::Button,
        "scheme",
        "VariantProps.color",
    );
    register_consumed_prop(
        &mut registry,
        BuiltinComponent::Button,
        "scheme",
        "VariantProps.color",
    );
    assert_eq!(registry.entries().len(), 1);
    assert_eq!(registry.entries()[0].prop, "scheme");
    assert_eq!(registry.entries()[0].ir_field, "VariantProps.color");
    registry.validate().expect("valid consumption");
}

#[test]
fn generated_view_prop_inventory_is_unique_and_valid() {
    let mut keys = std::collections::BTreeSet::new();
    for definition in VIEW_PROP_INVENTORY {
        let key = match definition.owner {
            ViewPropOwner::CommonStyle => ("CommonStyle", definition.prop),
            ViewPropOwner::Component(component) => (component.as_str(), definition.prop),
            ViewPropOwner::Item(item) => (match item {
                ViewItemKind::Tab => "Item:Tab",
                ViewItemKind::Accordion => "Item:Accordion",
                ViewItemKind::Carousel => "Item:Carousel",
                ViewItemKind::Option => "Item:Option",
                ViewItemKind::TableColumn => "Item:TableColumn",
                ViewItemKind::NavMenu => "Item:NavMenu",
                ViewItemKind::SideNav => "Item:SideNav",
                ViewItemKind::RailNav => "Item:RailNav",
                ViewItemKind::BottomBar => "Item:BottomBar",
                ViewItemKind::SvgPath => "Item:SvgPath",
            }, definition.prop),
        };
        assert!(keys.insert(key));
        assert!(!definition.ir_field.as_string().is_empty());
    }
}

#[test]
fn generated_view_prop_inventory_uses_the_ir_definition_type() {
    let definition: Option<&ViewPropDefinition> = VIEW_PROP_INVENTORY.first();
    assert!(definition.is_some());
    assert_eq!(VIEW_IR_SCHEMA_VERSION, 1);
}

#[test]
fn variant_props_are_declared_for_variant_components() {
    for component in [
        BuiltinComponent::Button,
        BuiltinComponent::IconButton,
        BuiltinComponent::Swap,
        BuiltinComponent::Input,
        BuiltinComponent::Select,
        BuiltinComponent::Card,
    ] {
        assert!(view_prop_declared(component, "variant"));
        assert!(view_prop_declared(component, "scheme"));
        assert!(view_prop_declared(component, "rounded"));
        if component != BuiltinComponent::Card {
            assert!(view_prop_declared(component, "size"));
        }
    }
    assert!(view_prop_declared(BuiltinComponent::Tabs, "variant"));
    assert!(view_prop_declared(BuiltinComponent::Tabs, "scheme"));
}

#[test]
fn form_props_are_declared_for_form_components() {
    for (component, props) in [
        (BuiltinComponent::Input, &["label", "placeholder", "bind"][..]),
        (BuiltinComponent::Select, &["label", "placeholder", "bind"]),
        (BuiltinComponent::Checkbox, &["bind", "checked"]),
        (BuiltinComponent::Toggle, &["bind", "checked"]),
        (BuiltinComponent::RadioGroup, &["bind", "orientation"]),
        (BuiltinComponent::Slider, &["bind", "min", "max", "step"]),
        (BuiltinComponent::Date, &["bind"]),
        (BuiltinComponent::DateRange, &["bind"]),
        (BuiltinComponent::Password, &["bind", "placeholder"]),
        (BuiltinComponent::Phone, &["bind", "country"]),
        (BuiltinComponent::Pin, &["bind", "length"]),
        (BuiltinComponent::Textarea, &["bind", "placeholder"]),
        (BuiltinComponent::Color, &["bind"]),
        (BuiltinComponent::Dropzone, &["multiple", "accept"]),
    ] {
        for prop in props {
            assert!(view_prop_declared(component, prop), "{component:?}.{prop}");
        }
    }
}

#[test]
fn media_props_are_declared_for_media_components() {
    for (component, props) in [
        (BuiltinComponent::Audio, &["src", ][..]),
        (BuiltinComponent::Video, &["src", "poster", ]),
        (BuiltinComponent::Iframe, &["src", "title", "sandbox", "allow", "allowFullscreen"]),
        (BuiltinComponent::Device, &["device", "zoom", "fit", "src"]),
        (BuiltinComponent::Image, &["src", "alt", "width", "height", "objectFit", "loading"]),
        (BuiltinComponent::Camera, &["facing", "resolution", "onCapture", "onError"]),
        (BuiltinComponent::Microphone, &["onError"]),
    ] {
        for prop in props {
            assert!(view_prop_declared(component, prop), "{component:?}.{prop}");
        }
    }
}

#[test]
fn navigation_props_are_declared_for_navigation_components() {
    for (component, props) in [
        (BuiltinComponent::AppBar, &["position", "floating", "bordered", "blurred", "hideOnScroll", "dockOnScroll"][..]),
        (BuiltinComponent::Footer, &["bordered", "blurred", "boxed"]),
        (BuiltinComponent::BottomBar, &["floating", "bordered", "blurred", "boxed"]),
        (BuiltinComponent::NavMenu, &["variant", "scheme", "size", ]),
        (BuiltinComponent::SideNav, &["variant", "scheme", "size", "wide"]),
        (BuiltinComponent::RailNav, &["variant", "scheme", "size"]),
        (BuiltinComponent::Sidebar, &["variant", "scheme", "size"]),
                (BuiltinComponent::Drawer, &["bind", "position", "disableOverlayClose", "hideCloseButton"]),
    ] {
        for prop in props {
            assert!(view_prop_declared(component, prop), "{component:?}.{prop}");
        }
    }
}

#[test]
fn chart_props_are_declared_for_chart_components() {
    for (component, props) in [
        (BuiltinComponent::Candlestick, &["data", "stream", "upColor", "downColor", "maxPoints"][..]),
        (BuiltinComponent::ArcChart, &["data", "size", "palette", "centerText", "centerValue", "showGlow"]),
        (BuiltinComponent::AreaChart, &["data", "series", "curve", "strokeWidth", "fillOpacity", "stacked", "showPoints"]),
        (BuiltinComponent::BarChart, &["data", "series", "grouped", "stacked", "showValues", "barRadius"]),
        (BuiltinComponent::LineChart, &["data", "series", "curve", "strokeWidth", "pointRadius", "hidePoints"]),
        (BuiltinComponent::PieChart, &["data", "donut", "donutWidth", "centerLabel", "centerValue", "padAngle"]),
    ] {
        for prop in props {
            assert!(view_prop_declared(component, prop), "{component:?}.{prop}");
        }
    }
}

#[test]
fn structural_and_item_props_are_declared() {
    for (component, props) in [
        (BuiltinComponent::Tabs, &["position"][..]),
        (BuiltinComponent::Tab, &["id", "label", "i18n"]),
        (BuiltinComponent::Stepper, &["position"]),
        (BuiltinComponent::Step, &["id", "label"]),
        (BuiltinComponent::Accordion, &["id", "title", "defaultOpen"]),
        (BuiltinComponent::Carousel, &["id", "title", "slidesPerView", "autoplay"]),
        (BuiltinComponent::Option, &["value", "label", "description"]),
        (BuiltinComponent::Table, &["field", "label", "align", "width"]),
        (BuiltinComponent::NavMenu, &["label", "i18n", "description", "descriptionI18n", "href"]),
        (BuiltinComponent::SideNav, &["label", "i18n", "description", "href"]),
        (BuiltinComponent::RailNav, &["label", "i18n", "href"]),
        (BuiltinComponent::BottomBar, &["label", "href"]),
        (BuiltinComponent::Svg, &["viewBox"]),
        (BuiltinComponent::Path, &["d", "fill"]),
    ] {
        for prop in props {
            assert!(view_prop_declared(component, prop), "{component:?}.{prop}");
        }
    }
}

#[test]
fn common_style_props_are_declared_for_visual_components() {
    for component in [
        BuiltinComponent::Box,
        BuiltinComponent::Section,
        BuiltinComponent::Flex,
        BuiltinComponent::Grid,
        BuiltinComponent::Card,
    ] {
        assert!(view_prop_declared(component, "p"));
        assert!(view_prop_declared(component, "rounded"));
        assert!(view_prop_declared(component, "show"));
    }
    assert!(component_prop_contract(BuiltinComponent::Button, "loading").is_some());
}
