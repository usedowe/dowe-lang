fn is_known_layout_and_data_prop(component: BuiltinComponent, name: &str) -> bool {
    match component {
            BuiltinComponent::Box => {
                matches!(
                    name,
                    "bg"
                        | "color"
                        | "centerX"
                        | "centerY"
                        | "cover"
                        | "overlay"
                        | "animation"
                        | "colSpan"
                        | "rowSpan"
                        | "flex"
                        | "position"
                        | "top"
                        | "right"
                        | "bottom"
                        | "left"
                        | "onClick"
                )
            }
            BuiltinComponent::Section => {
                matches!(
                    name,
                    "bg" | "color"
                        | "background"
                        | "centerX"
                        | "centerY"
                        | "gap"
                        | "boxed"
                        | "cover"
                        | "overlay"
                        | "animation"
                        | "shadow"
                        | "shadowColor"
                        | "borderColor"
                        | "colSpan"
                        | "rowSpan"
                        | "flex"
                )
            }
            BuiltinComponent::Flex => matches!(name, "justify" | "align" | "gap" | "flex"),
            BuiltinComponent::Grid => {
                matches!(name, "columns" | "rows" | "justify" | "align" | "gap" | "flex")
            }
            BuiltinComponent::Input | BuiltinComponent::Select => {
                matches!(
                    name,
                    "bind"
                        | "variant"
                        | "scheme"
                        | "size"
                        | "label"
                        | "placeholder"
                        | "labelFloating"
                        | "iconStart"
                        | "iconEnd"
                )
            }
            BuiltinComponent::Code => {
                matches!(
                    name,
                    "lines"
                        | "content"
                        | "language"
                        | "copyLabel"
                        | "copiedLabel"
                        | "template"
                        | "variant"
                        | "scheme"
                )
            }
            BuiltinComponent::Video => {
                matches!(
                    name,
                    "src" | "poster" | "autoplay" | "aspect" | "variant" | "scheme"
                )
            }
            BuiltinComponent::Iframe => {
                matches!(
                    name,
                    "src" | "title" | "loading" | "allow" | "sandbox" | "allowFullscreen"
                )
            }
            BuiltinComponent::Device => matches!(name, "device"),
            BuiltinComponent::Canvas => {
                matches!(
                    name,
                    "scene"
                        | "viewWidth"
                        | "viewHeight"
                        | "fit"
                        | "fps"
                        | "autoplay"
                        | "background"
                        | "pixelated"
                        | "label"
                        | "onPointer"
                        | "onKey"
                        | "onMotion"
                        | "motionRate"
                        | "bg"
                        | "color"
                        | "cover"
                        | "overlay"
                        | "animation"
                        | "colSpan"
                        | "rowSpan"
                        | "minW"
                        | "minH"
                        | "maxW"
                        | "maxH"
                )
            }
            BuiltinComponent::Audio => {
                matches!(
                    name,
                    "src" | "subtitle" | "avatarSrc" | "variant" | "scheme" | "color"
                )
            }
            BuiltinComponent::Camera => matches!(
                name,
                "facing" | "label" | "disabled" | "onStart" | "onCapture" | "onError" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::Microphone => matches!(
                name,
                "label" | "maxDuration" | "disabled" | "onStart" | "onStop" | "onError" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::Image => matches!(
                name,
                "src"
                    | "alt"
                    | "aspect"
                    | "objectFit"
                    | "loading"
                    | "hideControls"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Candlestick => {
                matches!(
                    name,
                    "data"
                        | "stream"
                        | "variant"
                        | "scheme"
                        | "upColor"
                        | "downColor"
                        | "emptyLabel"
                        | "maxPoints"
                )
            }
            BuiltinComponent::Diagram => {
                matches!(
                    name,
                    "nodes"
                        | "edges"
                        | "fitView"
                        | "panOnDrag"
                        | "zoomOnScroll"
                        | "minimap"
                        | "controls"
                        | "showGrid"
                        | "emptyLabel"
                        | "onNodeClick"
                        | "onNodeDrag"
                        | "onConnect"
                )
            }
            BuiltinComponent::ArcChart => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "centerText"
                        | "centerValue"
                        | "thickness"
                        | "gap"
                        | "startAngle"
                        | "endAngle"
                        | "showInlineLabels"
                        | "hideValues"
                        | "showGlow"
                )
            }
            BuiltinComponent::AreaChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "curve"
                        | "strokeWidth"
                        | "fillOpacity"
                        | "stacked"
                        | "hideLine"
                        | "showPoints"
                        | "hideGrid"
                        | "hideXAxis"
                        | "hideYAxis"
                        | "showGlow"
                )
            }
            BuiltinComponent::BarChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "grouped"
                        | "stacked"
                        | "showValues"
                        | "barRadius"
                        | "hideGrid"
                        | "showGlow"
                )
            }
            BuiltinComponent::LineChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "curve"
                        | "strokeWidth"
                        | "pointRadius"
                        | "hidePoints"
                        | "hideGrid"
                        | "hideXAxis"
                        | "hideYAxis"
                        | "showGradientFill"
                        | "showGlow"
                )
            }
            BuiltinComponent::PieChart => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "donut"
                        | "donutWidth"
                        | "centerLabel"
                        | "centerValue"
                        | "startAngle"
                        | "padAngle"
                        | "hideLabels"
                        | "hideValues"
                        | "hidePercentages"
                        | "showGlow"
                )
            }
        _ => false,
    }
}
