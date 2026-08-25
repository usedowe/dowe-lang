fn is_known_shell_and_feedback_prop(component: BuiltinComponent, name: &str) -> bool {
    match component {
        BuiltinComponent::Alert => {
            matches!(
                name,
                "type" | "message" | "visible" | "onClose" | "variant" | "scheme"
            )
        }
        BuiltinComponent::Card => {
            matches!(
                name,
                "variant"
                    | "scheme"
                    | "cover"
                    | "overlay"
                    | "animation"
                    | "colSpan"
                    | "rowSpan"
                    | "flex"
                    | "onClick"
            )
        }
        BuiltinComponent::Svg => {
            matches!(
                name,
                "id" | "show" | "viewBox" | "data" | "color" | "w" | "h"
            )
        }
        BuiltinComponent::Icon => {
            matches!(
                name,
                "name" | "fill" | "stroke" | "id" | "show" | "w" | "h"
            )
        }
        BuiltinComponent::Path => matches!(name, "d" | "fill" | "fillRule" | "transform"),
        BuiltinComponent::AppBar => {
            matches!(
                name,
                "variant"
                    | "scheme"
                    | "position"
                    | "bordered"
                    | "blurred"
                    | "boxed"
                    | "floating"
                    | "hideOnScroll"
                    | "dockOnScroll"
                    | "bind"
            )
        }
        BuiltinComponent::BottomBar => {
            matches!(
                name,
                "variant" | "scheme" | "bordered" | "blurred" | "boxed" | "floating"
            )
        }
        BuiltinComponent::Footer => {
            matches!(
                name,
                "variant" | "scheme" | "bordered" | "blurred" | "boxed"
            )
        }
        BuiltinComponent::SideNav => matches!(name, "variant" | "scheme" | "size" | "wide"),
        BuiltinComponent::RailNav => {
            matches!(name, "variant" | "scheme" | "size" | "showLabels")
        }
        BuiltinComponent::Sidebar => matches!(name, "variant" | "scheme" | "color"),
        BuiltinComponent::NavMenu => {
            matches!(name, "variant" | "scheme" | "size" | "color")
        }
        BuiltinComponent::Scaffold => matches!(name, "boxed"),
        BuiltinComponent::Splash => matches!(name, "bind"),
        BuiltinComponent::Tabs => matches!(name, "variant" | "scheme" | "position"),
        BuiltinComponent::Tab => matches!(name, "id" | "label"),
        BuiltinComponent::Stepper => matches!(name, "scheme" | "orientation"),
        BuiltinComponent::Step => matches!(name, "id" | "label"),
        BuiltinComponent::Drawer => matches!(
            name,
            "bind" | "position" | "variant" | "scheme" | "disableOverlayClose" | "hideCloseButton"
        ),
        BuiltinComponent::Avatar => matches!(
            name,
            "src"
                | "name"
                | "alt"
                | "href"
                | "navigate"
                | "history"
                | "target"
                | "externalMode"
                | "onClick"
                | "variant"
                | "scheme"
                | "size"
                | "status"
                | "bordered"
                | "color"
        ),
        BuiltinComponent::Badge => {
            matches!(name, "text" | "variant" | "scheme" | "position" | "color")
        }
        BuiltinComponent::Chip => {
            matches!(
                name,
                "onClose"
                    | "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
                    | "startIcon"
                    | "endIcon"
            )
        }
        BuiltinComponent::Skeleton => matches!(name, "variant" | "animation"),
        BuiltinComponent::Modal => matches!(
            name,
            "bind"
                | "onClose"
                | "variant"
                | "scheme"
                | "disableOverlayClose"
                | "hideCloseButton"
                | "color"
        ),
        BuiltinComponent::AlertDialog => matches!(
            name,
            "bind"
                | "title"
                | "description"
                | "confirmText"
                | "cancelText"
                | "onConfirm"
                | "onCancel"
                | "variant"
                | "scheme"
                | "loading"
                | "color"
        ),
        BuiltinComponent::Tooltip => {
            matches!(name, "label" | "position" | "variant" | "scheme" | "color")
        }
        BuiltinComponent::Toast => matches!(
            name,
            "source"
                | "type"
                | "title"
                | "description"
                | "position"
                | "variant"
                | "scheme"
                | "showIcon"
                | "color"
        ),
        BuiltinComponent::Dropdown => matches!(name, "scheme" | "color"),
        BuiltinComponent::Command => matches!(
            name,
            "bind"
                | "placeholder"
                | "emptyText"
                | "closeText"
                | "navigateText"
                | "selectText"
                | "toggleText"
                | "shortcut"
                | "disableGlobalShortcut"
                | "showFooter"
                | "variant"
                | "scheme"
                | "color"
        ),
        _ => false,
    }
}
