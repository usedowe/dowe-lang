
fn component_description(name: &str) -> &'static str {
    match name {
        "Box" | "Section" | "Flex" | "Grid" | "Card" | "Scaffold" => {
            "Built-in cross-platform layout component lowered by Dowe for every enabled Views target."
        }
        "Brand" => {
            "Built-in cross-platform identity component for arbitrary logo children with optional navigation."
        }
        "Banner" => {
            "Built-in cross-platform external banner component for arbitrary visual children with required HTTPS navigation."
        }
        "Splash" => {
            "Direct layout or page boundary that replaces normal content while its bound boolean Signal is true."
        }
        "AppBar" | "Footer" | "BottomBar" | "SideNav" | "RailNav" | "Sidebar" | "NavMenu"
        | "Tabs" | "tab" | "Stepper" | "step" | "Drawer" => {
            "Built-in cross-platform navigation and application-shell component."
        }
        "Draw" => {
            "Built-in cross-platform editable drawing surface with Signal-backed layers, selection, deletion, and layer events."
        }
        "Input" | "Select" | "Option" | "Slider" | "Dropzone" | "ComboBox" | "comboOption"
        | "CsvField" | "csvColumn" | "DragDrop" | "dragGroup" | "dragItem" | "Editor"
        | "ImageCropper" | "Password" | "Phone" | "Pin" | "Textarea" | "Checkbox" | "Color"
        | "Date" | "DateRange" | "RadioGroup" | "RadioCard" | "Toggle" | "ToggleGroup" => {
            "Built-in cross-platform form and interaction component."
        }
        "Code" | "Video" | "Iframe" | "Device" | "Audio" | "Camera" | "Microphone" | "Image"
        | "Canvas" | "Icon" | "Svg" | "Path" | "Candlestick" | "ArcChart" | "AreaChart"
        | "BarChart" | "LineChart" | "PieChart" | "Table" | "Tree" => {
            "Built-in cross-platform media or data-display component."
        }
        _ => "Built-in Dowe Views component lowered to web, desktop, Android, and iOS targets.",
    }
}

fn component_children(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "Box" | "Section" | "Flex" | "Grid" | "Card" => &[("view components", "(zero or more)")],
        "Brand" => &[("view components", "(one or more identity children)")],
        "Banner" => &[("view components", "(one or more banner children)")],
        "Splash" => &[("view components", "(zero or more splash children)")],
        "Badge" | "Tooltip" | "Marquee" | "Collapsible" => &[("view components", "(one or more)")],
        "Button" | "Title" | "Text" => &[("\"text\"", "(one direct static string)")],
        "Chip" => &[
            ("start", "(optional Svg icon region)"),
            ("\"text\"", "(one direct static string)"),
            ("end", "(optional Svg icon region)"),
        ],
        "Select" => &[
            ("Option", "(one or more option entries)"),
            ("validate", "(zero or more ordered validation rules)"),
        ],
        "Input" | "Date" | "Pin" | "Phone" | "Checkbox" => {
            &[("validate", "(zero or more ordered validation rules)")]
        }
        "ComboBox" => &[("comboOption", "(one or more option entries)")],
        "CsvField" => &[("csvColumn", "(one or more column entries)")],
        "DragDrop" => &[
            ("dragItem", "(direct draggable entry)"),
            ("dragGroup", "(group of draggable entries)"),
        ],
        "dragGroup" => &[("dragItem", "(one or more draggable entries)")],
        "Table" => &[("column", "(one or more column entries)")],
        "Svg" => &[(
            "Path",
            "(one or more static path entries, or none with runtime data)",
        )],
        "AppBar" | "Footer" => &[
            ("top", "(optional full-width region)"),
            ("start", "(optional region)"),
            ("centerX", "(optional region)"),
            ("end", "(optional region)"),
            ("bottom", "(optional full-width region)"),
        ],
        "BottomBar" => &[("tab", "(one or more navigation tabs with one Icon child)")],
        "NavMenu" => &[
            ("item", "(navigation entry)"),
            ("submenu", "(nested navigation entries)"),
            ("megamenu", "(navigation entry with a content region)"),
        ],
        "SideNav" => &[
            ("header", "(optional heading entry)"),
            ("item", "(navigation entry)"),
            ("divider", "(optional separator)"),
            ("submenu", "(nested navigation entries)"),
        ],
        "RailNav" => &[
            ("item", "(icon navigation entry)"),
            ("divider", "(optional separator)"),
        ],
        "Sidebar" => &[
            ("header", "(optional region)"),
            ("body", "(required region)"),
            ("footer", "(optional region)"),
        ],
        "Scaffold" => &[
            ("appBar", "(optional region)"),
            ("start", "(optional region)"),
            ("main", "(required region)"),
            ("end", "(optional region)"),
            ("bottomBar", "(optional region)"),
            ("overlays", "(optional region)"),
        ],
        "Drawer" => &[
            ("header", "(optional region)"),
            ("body", "(required region)"),
            ("footer", "(optional region)"),
            (
                "view components",
                "(also accepted directly as body content)",
            ),
        ],
        "Modal" => &[
            ("header", "(optional region)"),
            ("view components", "(required body content)"),
            ("footer", "(optional region)"),
        ],
        "Avatar" => &[("icon", "(optional region with one Svg or Icon child)")],
        "Dropdown" => &[
            ("trigger", "(required region)"),
            ("header", "(optional region)"),
            ("item", "(menu entry)"),
            ("divider", "(optional separator)"),
            ("footer", "(optional region)"),
        ],
        "Command" => &[
            ("item", "(command entry)"),
            ("group", "(group of command entries)"),
        ],
        "AvatarGroup" => &[("item", "(static entry; optional with the items prop)")],
        "TypeWriter" | "ToggleGroup" | "Accordion" | "RadioGroup" | "RadioCard" => {
            &[("item", "(one or more entries)")]
        }
        "RichText" => &[("mark", "(one or more rich-text runs)")],
        "Map" => &[
            ("marker", "(map marker entry)"),
            ("waypoint", "(route waypoint entry)"),
        ],
        "Carousel" => &[("slide", "(one or more slide entries)")],
        "Fab" => &[("fabAction", "(zero or more secondary actions)")],
        "Tabs" => &[("tab", "(one or more tab entries)")],
        "tab" => &[("view components", "(one or more)")],
        "Stepper" => &[("step", "(one or more ordered step entries)")],
        "step" => &[("view components", "(one or more)")],
        _ => &[],
    }
}


