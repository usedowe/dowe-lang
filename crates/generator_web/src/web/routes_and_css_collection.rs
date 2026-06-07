fn design_css_for_fonts(
    used_fonts: &BTreeSet<FontFamily>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    let fonts = font_config.effective_families(used_fonts);
    let mut css = String::new();
    css.push_str(":root{");
    append_theme_variables(&mut css, design_config.default_theme());
    for font in &fonts {
        css.push_str(&format!(
            "--dowe-font-{}:{};",
            font.as_str(),
            font_stack(*font)
        ));
    }
    css.push('}');
    for theme in &design_config.themes {
        if theme.name != design_config.default_theme {
            css.push_str(&format!(
                "[data-dowe-theme=\"{}\"]{{",
                escape_css_string(&theme.name)
            ));
            append_theme_variables(&mut css, theme);
            css.push('}');
        }
    }
    for font in &fonts {
        let entry = font.catalog_entry();
        if entry.package_assets {
            for weight in entry.weights {
                css.push_str(&format!(
                    "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:{};src:url(\"/fonts/{}/{}.ttf\") format(\"truetype\");font-display:swap;}}",
                    entry.display_name,
                    weight.numeric_weight,
                    font.as_str(),
                    weight.asset_stem
                ));
            }
        } else if *font != FontFamily::System {
            css.push_str(&format!(
                "@font-face{{font-family:\"Dowe {}\";font-style:normal;font-weight:300 800;src:local(\"{}\");font-display:swap;}}",
                entry.display_name, entry.display_name
            ));
        }
    }
    css.push_str(&format!("*,::before,::after{{box-sizing:border-box;}}html{{font-family:var(--dowe-font-{});line-height:1.5;-webkit-text-size-adjust:100%;tab-size:4;}}body{{margin:0;min-width:100%;min-height:100vh;background:var(--dowe-background);color:var(--dowe-onBackground);font-family:var(--dowe-font-{});}}p,h1,h2,h3,h4,h5,h6{{margin:0;font-size:inherit;font-weight:inherit;}}a{{color:inherit;text-decoration:inherit;}}button,input,textarea,select{{font:inherit;color:inherit;margin:0;}}button{{text-transform:none;}}img,svg,video,canvas{{display:block;max-width:100%;}}", font_config.default_family.as_str(), font_config.default_family.as_str()));
    css.push_str("[hidden]{display:none!important;}");
    append_visibility_css(&mut css);
    css.push_str(".box{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));}");
    css.push_str(".section{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));}");
    css.push_str(".flex{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));}");
    css.push_str(".grid{--dowe-component-display:grid;display:var(--dowe-show,var(--dowe-component-display));}");
    css.push_str(".card{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));border-radius:var(--dowe-radiusBox);overflow:hidden;}");
    css.push_str(".has-background{overflow:hidden;background-size:cover;}");
    css.push_str(".has-cover{position:relative;overflow:hidden;background-size:cover;background-position:center;background-repeat:no-repeat;}");
    css.push_str(".has-overlay::before{content:\"\";position:absolute;inset:0;z-index:0;pointer-events:none;}");
    css.push_str(".has-cover>*{position:relative;z-index:1;}");
    css.push_str(".button{--dowe-component-display:inline-flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:center;border-radius:var(--dowe-radiusUi);border:0;background:transparent;font:inherit;text-decoration:none;}");
    css.push_str(".svg{--dowe-component-display:inline-block;display:var(--dowe-show,var(--dowe-component-display));vertical-align:-0.125em;flex-shrink:0;}");
    css.push_str(".video{--dowe-component-display:block;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:100%;overflow:hidden;aspect-ratio:16/9;border-radius:var(--dowe-radiusBox);}.video.horizontal{aspect-ratio:16/9;}.video.vertical{aspect-ratio:9/16;}.video.square{aspect-ratio:1/1;}.video-media{position:absolute;inset:0;width:100%;height:100%;object-fit:cover;border-radius:inherit;background:currentColor;}");
    css.push_str(".media{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:100%;gap:.75rem;border-radius:var(--dowe-radiusBox);padding:.375rem .75rem;user-select:none;}.media-button{display:inline-flex;width:2.5rem;height:2.5rem;flex:0 0 auto;align-items:center;justify-content:center;border:0;border-radius:9999px;background:rgba(255,255,255,.18);color:inherit;cursor:pointer;font:inherit;}.media-content{display:flex;min-width:0;flex:1;flex-direction:column;gap:.125rem;}.media-waveform{display:flex;width:100%;height:2rem;align-items:center;padding-top:.75rem;cursor:pointer;}.media-bars{display:flex;width:100%;height:100%;align-items:center;gap:.125rem;}.media-bar{min-height:.25rem;flex:1;border-radius:.125rem;background:currentColor;opacity:.3;transition:opacity 160ms ease,transform 160ms ease;}.media-bar.active{opacity:1;}.media-footer{display:flex;align-items:center;justify-content:space-between;gap:.75rem;font-size:.75rem;line-height:1rem;opacity:.72;}.media-time{flex:0 0 auto;font-weight:600;}.media-subtitle{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}.media-avatar{width:3rem;height:3rem;flex:0 0 auto;border-radius:9999px;object-fit:cover;}.image{--dowe-component-display:block;position:relative;display:var(--dowe-show,var(--dowe-component-display));overflow:hidden;border-radius:var(--dowe-radiusBox);}.image.horizontal{aspect-ratio:16/9;}.image.vertical{aspect-ratio:9/16;}.image.square{aspect-ratio:1/1;}.image.auto{aspect-ratio:auto;}.image-element{width:100%;height:100%;}.image.fit-cover .image-element{object-fit:cover;}.image.fit-contain .image-element{object-fit:contain;}.image.fit-fill .image-element{object-fit:fill;}.image.fit-none .image-element{object-fit:none;}.image-controls{position:absolute;inset-inline:0;bottom:0;z-index:2;display:flex;justify-content:flex-end;padding:.5rem;opacity:0;transition:opacity 160ms ease;}.image:hover .image-controls,.image:focus-within .image-controls{opacity:1;}.image-actions{display:flex;align-items:center;gap:.5rem;}.image-action{display:inline-flex;width:2rem;height:2rem;align-items:center;justify-content:center;border:1px solid rgba(255,255,255,.35);border-radius:9999px;background:rgba(15,23,42,.58);color:white;cursor:pointer;font:inherit;backdrop-filter:blur(8px);}");
    css.push_str(".candlestick{--dowe-component-display:block;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:100%;min-height:16rem;margin:0;overflow:hidden;border-radius:var(--dowe-radiusBox);}.candlestick-canvas{position:absolute;inset:0;width:100%;height:100%;}.candlestick-empty{position:absolute;inset:0;display:flex;align-items:center;justify-content:center;padding:1rem;text-align:center;font-size:0.875rem;opacity:0.72;}.candlestick.has-data .candlestick-empty{display:none;}");
    css.push_str(".table-wrapper{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;gap:0.5rem;width:100%;}.table-container{position:relative;width:100%;overflow-x:auto;}.table{position:relative;width:100%;border-collapse:separate;border-spacing:0;text-align:start;border-radius:var(--dowe-radiusBox);}.table-header{white-space:nowrap;background:var(--dowe-softMuted);color:currentColor;font-size:0.875rem;font-weight:600;}.table-head{padding:0.75rem 1rem;text-align:start;font-size:0.875rem;font-weight:600;}.table-head:first-child{padding-left:1.5rem;border-start-start-radius:var(--dowe-radiusBox);}.table-head:last-child{border-start-end-radius:var(--dowe-radiusBox);}.table-head-content{display:flex;align-items:center;gap:0.25rem;}.table-head-label{flex-shrink:0;}.table-body tr{transition:background-color 150ms ease;}.table-body td{padding:1rem;text-align:start;font-size:0.875rem;white-space:nowrap;}.table-body td:first-child{padding-left:1.5rem;}.table.has-dividers .table-body tr+tr td{border-top:1px solid currentColor;}.table.has-dividers .table-body td{border-color:rgba(127,127,127,0.28);}.table.is-striped .table-body tr:nth-child(even){background:rgba(127,127,127,0.12);}.table.is-striped .table-body tr:hover{background:rgba(127,127,127,0.18);}.table.is-bordered{border:1px solid rgba(127,127,127,0.28);overflow:hidden;}.table.is-bordered .table-body td{border-inline-end:1px solid rgba(127,127,127,0.28);}.table.is-bordered .table-body td:last-child{border-inline-end:0;}.table.is-sm .table-head{padding:0.5rem 0.75rem;font-size:0.75rem;}.table.is-sm .table-head:first-child{padding-left:1rem;}.table.is-sm .table-body td{padding:0.5rem 0.75rem;font-size:0.75rem;}.table.is-sm .table-body td:first-child{padding-left:1rem;}.table.is-md .table-head{padding:0.75rem 1rem;font-size:0.875rem;}.table.is-md .table-body td{padding:1rem;font-size:0.875rem;}.table.is-lg .table-head{padding:1rem 1.25rem;font-size:1rem;}.table.is-lg .table-head:first-child{padding-left:1.75rem;}.table.is-lg .table-body td{padding:1.25rem;font-size:1rem;}.table.is-lg .table-body td:first-child{padding-left:1.75rem;}.table-empty-row{border:0;}.table-empty-cell{border:0!important;padding:3rem 1rem!important;}.empty-state{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:1rem;text-align:center;}.empty-content{display:flex;flex-direction:column;gap:0.25rem;}.empty-title{font-size:1.125rem;line-height:1.75rem;font-weight:600;color:inherit;}.empty-description{font-size:0.875rem;line-height:1.25rem;opacity:0.72;}");
    css.push_str(".divider{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));flex-shrink:0;background-color:var(--dowe-muted);color:var(--dowe-muted);}.divider-horizontal{width:100%;height:1px;}.divider-vertical{width:1px;height:100%;align-self:stretch;}");
    css.push_str(".code-block{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));overflow:hidden;border-radius:var(--dowe-radiusBox);}.code-toolbar{display:flex;align-items:center;justify-content:space-between;gap:0.75rem;padding:0.625rem 0.75rem;border-bottom:1px solid currentColor;font-size:0.75rem;font-weight:600;opacity:0.88;}.code-language{text-transform:uppercase;letter-spacing:0.08em;}.code-copy{border:0;border-radius:var(--dowe-radiusUi);padding:0.25rem 0.5rem;background:transparent;color:inherit;cursor:pointer;font:inherit;}.code-copy:hover{background:rgba(127,127,127,0.16);}.code-pre{margin:0;overflow-x:auto;padding:1rem;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,\"Liberation Mono\",monospace;font-size:0.875rem;line-height:1.6;tab-size:2;}.code-token-keyword{color:var(--dowe-primary);}.code-token-type{color:var(--dowe-info);}.code-token-string{color:var(--dowe-success);}.code-token-number{color:var(--dowe-warning);}.code-token-attribute{color:var(--dowe-tertiary);}.code-token-comment{color:var(--dowe-muted);}.code-token-punctuation{color:var(--dowe-danger);}");
    css.push_str(".field{display:flex;flex-direction:column;gap:0.5rem;width:100%;}.field-label{font-size:0.875rem;line-height:1.25rem;font-weight:600;}.field>.control,.field>.select{width:100%;}");
    css.push_str(&format!(
        ".control{{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:100%;min-height:{};gap:0.5rem;border-radius:var(--dowe-radiusUi);}}.control.is-floating{{padding-top:0.75rem;}}.control-label{{position:absolute;left:{};top:50%;max-width:calc(100% - {});overflow:hidden;text-overflow:ellipsis;white-space:nowrap;transform:translateY(-50%);font-size:1em;line-height:1;pointer-events:none;transition:top 160ms ease,transform 160ms ease,font-size 160ms ease;}}.control.is-floating:focus-within .control-label,.control.is-floating:has(.input:not(:placeholder-shown)) .control-label,.control.is-floating.has-value .control-label,.control.is-floating.is-open .control-label{{top:0.25rem;transform:translateY(0);font-size:0.75rem;}}.flex>.control,.flex>.field,.flex>.select{{flex:1 1 0;min-width:0;}}",
        scale_rem(INPUT_MIN_HEIGHT),
        scale_rem(INPUT_HORIZONTAL_PADDING),
        scale_rem(ScaleValue::from_half_steps(INPUT_HORIZONTAL_PADDING.0 * 2))
    ));
    css.push_str(&format!(
        ".input{{box-sizing:border-box;flex:1;width:100%;min-width:0;min-height:{};padding:0 {};border:0;background:transparent;color:inherit;font-family:inherit;font-size:{};line-height:{};font-weight:400;outline:0;}}.control.is-floating .input::placeholder{{opacity:0;}}.control.is-floating:focus-within .input::placeholder{{opacity:1;}}",
        scale_rem(INPUT_MIN_HEIGHT),
        scale_rem(INPUT_HORIZONTAL_PADDING),
        text_size_css(INPUT_TEXT_SIZE),
        text_line_css(INPUT_TEXT_SIZE)
    ));
    css.push_str(".select{position:relative;width:100%;}.select-control{border:0;text-align:left;cursor:pointer;font:inherit;}.select-value{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}.select-control.is-floating:not(.is-open):not(.has-value) .select-value{visibility:hidden;}.select-arrow{width:1em;height:1em;flex:0 0 auto;color:currentColor;transition:transform 160ms ease;}.select-control.is-open .select-arrow{transform:rotate(180deg);}.select-popover{position:fixed;z-index:9999;display:flex;flex-direction:column;max-height:14rem;overflow-y:auto;padding:0.5rem;gap:0.25rem;background:var(--dowe-background);color:var(--dowe-onBackground);border-radius:var(--dowe-radiusBox);box-shadow:0 12px 32px rgba(15,23,42,0.18);opacity:0;visibility:hidden;pointer-events:none;transform:translateY(-0.25rem) scale(0.98);transform-origin:top center;transition:opacity 160ms ease,transform 160ms ease,visibility 160ms ease;}.select-popover.is-above{transform:translateY(0.25rem) scale(0.98);transform-origin:bottom center;}.select-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);}.select-option{display:flex;flex-direction:column;align-items:flex-start;gap:0.125rem;width:100%;border:0;border-radius:calc(var(--dowe-radiusUi) * 0.75);padding:0.5rem 0.625rem;background:transparent;color:inherit;text-align:left;cursor:pointer;}.select-option:hover,.select-option.is-focused{background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);}.select-option.is-selected{background:var(--dowe-softPrimary);color:var(--dowe-onSoftPrimary);}.select-option-label{font-weight:500;}.select-option-description{font-size:0.8125rem;opacity:0.72;}");
    css.push_str(".checkbox,.toggle{--dowe-component-display:inline-flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:max-content;gap:.5rem;cursor:pointer;}.checkbox-input{position:relative;width:1.25rem;height:1.25rem;min-width:1.25rem;appearance:none;border:2px solid var(--dowe-muted);border-radius:var(--dowe-radiusUi);outline:0;transition:background-color 160ms ease,border-color 160ms ease;}.checkbox-input:checked{background:currentColor;border-color:currentColor;}.checkbox-input:checked::after{content:\"\";position:absolute;inset:.15rem;background:white;clip-path:polygon(14% 50%,0 64%,40% 100%,100% 18%,84% 6%,38% 70%);}.label-md,.label{font-size:.875rem;line-height:1.25rem;}.toggle-input{position:relative;width:2.5rem;height:1.5rem;appearance:none;border:0;border-radius:9999px;background:var(--dowe-muted);outline:0;transition:background-color 160ms ease;}.toggle-input::before{content:\"\";position:absolute;left:.25rem;top:.25rem;width:1rem;height:1rem;border-radius:9999px;background:white;box-shadow:0 1px 4px rgba(15,23,42,.22);transition:transform 160ms ease;}.toggle-input:checked{background:currentColor;}.toggle-input:checked::before{transform:translateX(1rem);}.toggle-label-left,.toggle-label-right{color:var(--dowe-muted);transition:color 160ms ease;}.toggle-label-left.is-active,.toggle-label-right.is-active{color:var(--dowe-onMuted);}.checkbox-input.is-primary,.radio.is-primary,.toggle-input.is-primary{color:var(--dowe-primary);}.checkbox-input.is-secondary,.radio.is-secondary,.toggle-input.is-secondary{color:var(--dowe-secondary);}.checkbox-input.is-tertiary,.radio.is-tertiary,.toggle-input.is-tertiary{color:var(--dowe-tertiary);}.checkbox-input.is-muted,.radio.is-muted,.toggle-input.is-muted{color:var(--dowe-onMuted);}.checkbox-input.is-success,.radio.is-success,.toggle-input.is-success{color:var(--dowe-success);}.checkbox-input.is-info,.radio.is-info,.toggle-input.is-info{color:var(--dowe-info);}.checkbox-input.is-warning,.radio.is-warning,.toggle-input.is-warning{color:var(--dowe-warning);}.checkbox-input.is-danger,.radio.is-danger,.toggle-input.is-danger{color:var(--dowe-danger);}.color-field,.date-field,.date-range-field{min-height:2.5rem;padding:0 .75rem;}.color-input{width:2rem;height:2rem;border:0;padding:0;background:transparent;cursor:pointer;}.color-field-display{display:flex;min-width:0;flex:1;align-items:center;gap:.5rem;}.color-field-swatch{display:inline-flex;width:1.5rem;height:1.5rem;flex:0 0 auto;border-radius:var(--dowe-radiusUi);box-shadow:inset 0 0 0 1px rgba(127,127,127,.28);}.color-field-swatch.is-sm{width:1.25rem;height:1.25rem;}.color-field-swatch.is-lg{width:2rem;height:2rem;}.color-field-value{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.875rem;text-transform:uppercase;}.color-picker-values{display:flex;flex-direction:column;gap:.25rem;width:100%;padding:.5rem 0;}.color-picker-value-code{display:block;border-radius:var(--dowe-radiusUi);background:rgba(127,127,127,.12);padding:.25rem .5rem;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.75rem;}.date-input{min-width:0;flex:1;border:0;background:transparent;color:inherit;outline:0;}.date-range-inputs{display:flex;min-width:0;flex:1;align-items:center;gap:.5rem;}.date-range-separator{opacity:.64;}.radio-group{display:flex;flex-wrap:wrap;gap:1rem;justify-content:flex-start;}.radio-item{display:inline-flex;align-items:center;gap:.75rem;cursor:pointer;}.radio{position:relative;appearance:none;border:2px solid var(--dowe-muted);border-radius:9999px;cursor:pointer;outline:0;transition:border-color 160ms ease;}.radio:checked{border-color:currentColor;}.radio:checked::after{content:\"\";position:absolute;left:50%;top:50%;border-radius:9999px;background:currentColor;transform:translate(-50%,-50%);}.radio.is-sm{width:1rem;height:1rem;}.radio.is-sm:checked::after{width:.5rem;height:.5rem;}.radio.is-md{width:1.25rem;height:1.25rem;}.radio.is-md:checked::after{width:.75rem;height:.75rem;}.radio.is-lg{width:1.5rem;height:1.5rem;}.radio.is-lg:checked::after{width:.875rem;height:.875rem;}.field-help{font-size:.8125rem;line-height:1.125rem;color:var(--dowe-muted);}.field-help.is-error{color:var(--dowe-danger);}");
    css.push_str(".accordion{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;gap:.5rem;width:100%;}.accordion-item{overflow:hidden;border-radius:var(--dowe-radiusBox);transition:opacity 160ms ease;}.accordion-item.is-disabled{opacity:.5;pointer-events:none;}.accordion-header{display:flex;width:100%;align-items:center;justify-content:space-between;gap:.75rem;border:0;background:transparent;color:inherit;padding:.75rem 1rem;text-align:left;cursor:pointer;font:inherit;transition:background-color 160ms ease,color 160ms ease;}.accordion-start{display:flex;min-width:0;flex:1;align-items:center;gap:.75rem;}.accordion-label{font-size:.875rem;font-weight:600;line-height:1.25rem;}.accordion-end{display:flex;align-items:center;}.accordion-arrow{display:inline-flex;width:1.25rem;height:1.25rem;align-items:center;justify-content:center;transition:transform 160ms ease;}.accordion-header.is-open .accordion-arrow{transform:rotate(180deg);}.accordion-content{padding:.75rem 1rem;font-size:.875rem;line-height:1.5;}.carousel{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:100%;flex-direction:column;gap:1rem;}.carousel.is-vertical{flex-direction:row;align-items:center;}.carousel-header{display:flex;align-items:center;justify-content:space-between;margin-bottom:.5rem;}.carousel-title h2{margin:0;font-size:1.5rem;font-weight:800;line-height:1.2;}.carousel-viewport{position:relative;width:100%;min-height:12.5rem;overflow:hidden;touch-action:pan-y;}.carousel-container{display:flex;height:100%;transition:transform 240ms ease;will-change:transform;}.carousel.is-vertical .carousel-container{flex-direction:column;}.carousel-slide{min-width:100%;height:100%;flex:0 0 100%;user-select:none;}.carousel.is-vertical .carousel-slide{min-height:100%;}.carousel-controls{position:relative;z-index:1;display:flex;align-items:center;justify-content:center;gap:.5rem;}.carousel-control,.carousel-nav{display:inline-flex;align-items:center;justify-content:center;border:1px solid rgba(127,127,127,.25);border-radius:9999px;background:var(--dowe-surface);color:var(--dowe-onSurface);box-shadow:0 2px 10px rgba(15,23,42,.12);cursor:pointer;font:inherit;}.carousel-control{width:2rem;height:2rem;}.carousel-nav{position:absolute;top:50%;z-index:2;width:2.5rem;height:2.5rem;transform:translateY(-50%);opacity:0;pointer-events:none;transition:opacity 160ms ease;}.carousel:hover .carousel-nav{opacity:1;pointer-events:auto;}.carousel-nav.is-prev{left:1rem;}.carousel-nav.is-next{right:1rem;}.carousel-indicators{display:flex;align-items:center;justify-content:center;gap:.5rem;}.carousel-indicator{height:.5rem;width:2rem;border:0;border-radius:9999px;background:var(--dowe-muted);cursor:pointer;transition:width 160ms ease,transform 160ms ease,background-color 160ms ease;}.carousel-indicator.is-active{background:currentColor;}.carousel-indicator.is-sm{width:1.5rem;}.carousel-indicator.is-sm.is-active{width:2rem;}.carousel-indicator.is-md{width:2rem;}.carousel-indicator.is-md.is-active{width:3rem;}.carousel-indicator.is-lg{width:2.5rem;}.carousel-indicator.is-lg.is-active{width:4rem;}.carousel-indicator.is-dot{width:.625rem;height:.625rem;}.carousel-indicator.is-dot.is-active{width:.625rem;transform:scale(1.25);}.carousel-counter{font-size:.875rem;font-weight:600;opacity:.72;}");
    css.push_str(".grid>[data-dowe-each],.flex>[data-dowe-each]{display:contents;}");
    css.push_str(".alert{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:space-between;gap:0.75rem;padding:0.625rem 0.875rem;border-radius:var(--dowe-radiusUi);}");
    css.push_str(".alert[hidden]{display:none;}");
    css.push_str(".alert-close{display:inline-flex;align-items:center;justify-content:center;width:1.5rem;height:1.5rem;border:0;border-radius:999px;background:transparent;color:inherit;cursor:pointer;}");
    css.push_str(".avatar{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:center;overflow:visible;border:0;border-radius:9999px;text-decoration:none;line-height:1;font-weight:700;transition:opacity 180ms ease,transform 180ms ease;}.avatar.is-clickable{cursor:pointer;}.avatar.is-clickable:hover{opacity:.82;transform:scale(1.04);}.avatar.is-clickable:active{transform:scale(.96);}.avatar.is-bordered{box-shadow:0 0 0 3px currentColor;}.avatar-image{width:100%;height:100%;object-fit:cover;border-radius:inherit;}.avatar-icon{display:flex;width:60%;height:60%;align-items:center;justify-content:center;}.avatar-icon .svg{width:100%;height:100%;}.avatar-name{line-height:1;}.avatar-status{position:absolute;right:0;bottom:0;transform:translate(25%,25%);}.avatar-indicator{display:block;border:2px solid var(--dowe-background);border-radius:9999px;}.avatar-indicator.is-online{background:var(--dowe-success);}.avatar-indicator.is-offline{background:var(--dowe-muted);}.avatar-indicator.is-busy{background:var(--dowe-warning);}.avatar-indicator.is-away{background:var(--dowe-danger);}.avatar-xs{width:1.5rem;height:1.5rem;font-size:.75rem;}.avatar-xs .avatar-indicator{width:.5rem;height:.5rem;}.avatar-sm{width:2rem;height:2rem;font-size:.875rem;}.avatar-sm .avatar-indicator{width:.625rem;height:.625rem;}.avatar-md{width:2.5rem;height:2.5rem;font-size:1rem;}.avatar-md .avatar-indicator{width:.75rem;height:.75rem;}.avatar-lg{width:3rem;height:3rem;font-size:1.125rem;}.avatar-lg .avatar-indicator{width:.875rem;height:.875rem;}.avatar-xl{width:4rem;height:4rem;font-size:1.5rem;}.avatar-xl .avatar-indicator{width:1rem;height:1rem;}.badge{--dowe-component-display:inline-block;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:max-content;}.badge-content{position:absolute;display:flex;align-items:center;justify-content:center;min-width:1.25rem;height:1.25rem;padding:0 .25rem;border-radius:9999px;font-size:.75rem;font-weight:700;line-height:1;}.badge.is-top-left .badge-content{top:5%;left:5%;transform:translate(-50%,-50%);}.badge.is-top-right .badge-content{top:5%;right:5%;transform:translate(50%,-50%);}.badge.is-bottom-left .badge-content{bottom:5%;left:5%;transform:translate(-50%,50%);}.badge.is-bottom-right .badge-content{right:5%;bottom:5%;transform:translate(50%,50%);}.chip{--dowe-component-display:inline-flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:center;gap:.5rem;width:max-content;max-width:100%;border-radius:var(--dowe-radiusUi);font-weight:600;white-space:nowrap;}.chip-label{overflow:hidden;text-overflow:ellipsis;}.chip-icon{display:inline-flex;align-items:center;justify-content:center;}.chip-close{display:inline-flex;align-items:center;justify-content:center;width:1.25em;height:1.25em;border:0;border-radius:9999px;background:transparent;color:inherit;cursor:pointer;opacity:.72;}.chip-close:hover{opacity:1;}.chip-xs{min-height:1.25rem;padding:0 .75rem;font-size:.75rem;}.chip-sm{min-height:1.5rem;padding:0 .75rem;font-size:.75rem;}.chip-md{min-height:2rem;padding:0 1rem;font-size:.875rem;}.chip-lg{min-height:2.5rem;padding:0 1.25rem;font-size:1rem;}.chip-xl{min-height:3rem;padding:0 1.5rem;font-size:1.25rem;}.chip.has-close{padding-right:.375rem;}.skeleton{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));background:var(--dowe-muted);border-radius:var(--dowe-radius);will-change:background-position,opacity;}.skeleton.is-text{height:1rem;width:100%;}.skeleton.is-circular{aspect-ratio:1/1;border-radius:9999px;}.skeleton.is-rectangular{border-radius:0;}.skeleton.is-rounded{border-radius:var(--dowe-radiusBox);}.skeleton.is-wave{animation:dowe-skeleton-wave 1.8s ease-in-out infinite;background-image:linear-gradient(105deg,transparent 0 40%,var(--dowe-background) 50%,transparent 60% 100%);background-size:200% auto;background-repeat:no-repeat;background-position-x:-50%;}.skeleton.is-pulse{animation:dowe-skeleton-pulse 1.5s ease-in-out infinite;}.modal-dialog,.command-dialog{--dowe-component-display:flex;position:fixed;inset:0;z-index:60;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:center;padding:1rem;}.modal-overlay{position:absolute;inset:0;border:0;background:rgba(15,23,42,.48);cursor:pointer;}.modal,.command{position:relative;display:flex;flex-direction:column;max-width:min(95vw,36rem);max-height:95vh;overflow:hidden;border-radius:var(--dowe-radiusBox);box-shadow:0 24px 64px rgba(15,23,42,.24);}.modal-header,.modal-footer{display:flex;width:100%;align-items:center;justify-content:space-between;gap:1rem;padding:1rem 1.25rem;}.modal-body{display:flex;flex-direction:column;gap:1rem;overflow:auto;padding:1rem 1.25rem;}.modal-close{position:absolute;top:.5rem;right:.5rem;display:inline-flex;width:1.75rem;height:1.75rem;align-items:center;justify-content:center;border:0;border-radius:9999px;background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);font-size:1.25rem;line-height:1;cursor:pointer;}.alert-dialog-title{font-size:1.125rem;font-weight:700;line-height:1.5;margin:0;}.alert-dialog-description{margin:0;color:var(--dowe-onMuted);}.alert-dialog-actions{display:flex;gap:.75rem;justify-content:flex-end;width:100%;}.tooltip{--dowe-component-display:inline-flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:max-content;}.tooltip-popover{position:fixed;z-index:9999;display:flex;align-items:center;gap:.5rem;padding:.5rem .75rem;border-radius:var(--dowe-radiusBox);box-shadow:0 10px 24px rgba(15,23,42,.18);font-size:.875rem;line-height:1.25;white-space:nowrap;opacity:0;visibility:hidden;pointer-events:none;transition:opacity 160ms ease,visibility 160ms ease;}.tooltip-popover.is-active{opacity:1;visibility:visible;}.tooltip-arrow{position:absolute;width:.5rem;height:.5rem;transform:rotate(45deg);background:currentColor;}.tooltip-popover.position-top .tooltip-arrow{left:50%;bottom:-.25rem;transform:translateX(-50%) rotate(45deg);}.tooltip-popover.position-bottom .tooltip-arrow{left:50%;top:-.25rem;transform:translateX(-50%) rotate(45deg);}.tooltip-popover.position-start .tooltip-arrow{right:-.25rem;top:50%;transform:translateY(-50%) rotate(45deg);}.tooltip-popover.position-end .tooltip-arrow{left:-.25rem;top:50%;transform:translateY(-50%) rotate(45deg);}.toast{--dowe-component-display:flex;position:fixed;z-index:70;display:var(--dowe-show,var(--dowe-component-display));align-items:center;gap:.75rem;width:min(26rem,calc(100vw - 2rem));padding:1rem;border-radius:var(--dowe-radiusBox);box-shadow:0 16px 44px rgba(15,23,42,.22);}.toast.is-top-left{top:1rem;left:1rem;}.toast.is-top-right{top:1rem;right:1rem;}.toast.is-bottom-left{bottom:1rem;left:1rem;}.toast.is-bottom-right{right:1rem;bottom:1rem;}.toast-content{display:flex;min-width:0;flex:1;flex-direction:column;gap:.25rem;}.toast-title{font-size:.875rem;font-weight:700;line-height:1.25;}.toast-description{font-size:.875rem;line-height:1.5;opacity:.9;}.toast-close{display:inline-flex;width:1.5rem;height:1.5rem;align-items:center;justify-content:center;border:0;border-radius:9999px;background:transparent;color:inherit;cursor:pointer;}.dropdown{--dowe-component-display:inline-flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));width:max-content;}.dropdown-trigger{display:inline-flex;}.dropdown-popover{position:fixed;z-index:9999;display:flex;min-width:12rem;max-height:20rem;flex-direction:column;gap:.5rem;overflow:auto;padding:.5rem;border-radius:var(--dowe-radiusBox);box-shadow:0 12px 32px rgba(15,23,42,.18);opacity:0;visibility:hidden;pointer-events:none;transform:translateY(-.25rem) scale(.98);transition:opacity 160ms ease,transform 160ms ease,visibility 160ms ease;}.dropdown-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);}.dropdown-options,.command-group-items{display:flex;flex-direction:column;gap:.25rem;}.dropdown-divider{height:1px;margin:.25rem 0;background:var(--dowe-muted);}.dropdown-item,.command-item{display:flex;width:100%;align-items:center;gap:.625rem;border:0;border-radius:var(--dowe-radiusUi);background:transparent;color:inherit;padding:.5rem .625rem;text-align:left;text-decoration:none;cursor:pointer;font:inherit;}.dropdown-item:hover,.command-item:hover,.command-item.is-focused{background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);}.dropdown-item.is-disabled,.command-item.is-disabled{opacity:.5;cursor:not-allowed;}.dropdown-item-content,.command-item-content{display:flex;min-width:0;flex:1;flex-direction:column;}.dropdown-item-label,.command-item-label{font-size:.875rem;font-weight:600;line-height:1.25;}.dropdown-item-description,.command-item-description{font-size:.75rem;line-height:1rem;opacity:.72;}.dropdown-item-icon,.command-item-icon{display:flex;width:1.25rem;height:1.25rem;flex:0 0 auto;align-items:center;justify-content:center;}.command{width:min(38rem,95vw);background:var(--dowe-background);color:var(--dowe-onBackground);}.command-header{display:flex;align-items:center;gap:.75rem;padding:.75rem 1rem;border-bottom:1px solid rgba(127,127,127,.22);}.command-input{flex:1;min-width:0;border:0;background:transparent;color:inherit;outline:0;font:inherit;}.command-kbd{display:none;align-items:center;gap:.375rem;font-size:.75rem;opacity:.7;}.command-kbd kbd,.command-shortcuts kbd{border:1px solid rgba(127,127,127,.28);border-radius:.25rem;background:rgba(127,127,127,.16);padding:.125rem .375rem;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.6875rem;}.command-results{display:flex;max-height:20rem;flex-direction:column;gap:.25rem;overflow:auto;padding:.5rem;}.command-empty{padding:2rem;text-align:center;font-size:.875rem;opacity:.64;}.command-group{display:flex;flex-direction:column;gap:.25rem;}.command-group+.command-group{margin-top:.5rem;padding-top:.5rem;border-top:1px solid rgba(127,127,127,.18);}.command-group-label{display:flex;align-items:center;gap:.5rem;padding:.375rem .625rem;font-size:.75rem;font-weight:700;text-transform:uppercase;opacity:.72;}.command-shortcuts{display:flex;align-items:center;justify-content:space-between;gap:.75rem;padding:.625rem 1rem;border-top:1px solid rgba(127,127,127,.22);font-size:.75rem;opacity:.72;}@media (min-width:768px){.command-kbd{display:flex;}}@keyframes dowe-skeleton-wave{0%{background-position:150%;}100%{background-position:-50%;}}@keyframes dowe-skeleton-pulse{0%,100%{opacity:1;}50%{opacity:.4;}}");
    css.push_str(".appbar,.footer,.bottombar{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));width:100%;transition:all 180ms ease;}.appbar{position:relative;z-index:30;padding-top:env(safe-area-inset-top,0px);}.footer{margin-top:auto;}.bottombar{position:sticky;bottom:0;z-index:30;padding-bottom:env(safe-area-inset-bottom,0px);}.appbar.is-bordered{border-bottom:1px solid var(--dowe-muted);}.footer.is-bordered,.bottombar.is-bordered{border-top:1px solid var(--dowe-muted);}.appbar.is-blurred,.footer.is-blurred,.bottombar.is-blurred{backdrop-filter:blur(16px);}.appbar.is-floating{margin:1rem auto 0;width:calc(100% - 2rem);border-radius:var(--dowe-radiusBox);border:1px solid var(--dowe-muted);overflow:hidden;}.bottombar.is-floating{margin:0 auto 1rem;width:calc(100% - 2rem);border-radius:var(--dowe-radiusBox);border:1px solid var(--dowe-muted);overflow:hidden;}.appbar-content,.footer-content,.bottombar-content{display:flex;flex-flow:row nowrap;align-items:center;justify-content:space-between;position:relative;width:100%;min-height:3rem;padding:0 0.5rem;}.footer-content{flex-wrap:wrap;}.appbar-content.is-boxed,.footer-content.is-boxed,.bottombar-content.is-boxed{max-width:72rem;margin:0 auto;}.appbar-start,.appbar-center,.appbar-end,.footer-start,.footer-center,.footer-end,.bottombar-start,.bottombar-center,.bottombar-end{display:flex;flex-flow:row nowrap;align-items:center;min-width:0;}.appbar-start,.appbar-end{gap:0.75rem;padding:0.75rem;}.appbar-center{flex:1 1 0;justify-content:center;gap:0.75rem;padding:0.75rem;}.footer-start,.footer-end,.bottombar-start,.bottombar-end{gap:0.5rem;padding:0.5rem;}.footer-center,.bottombar-center{flex:1 1 0;justify-content:center;gap:0.5rem;padding:0.5rem;}");
    css.push_str(".scaffold{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));min-height:100vh;width:100%;flex-direction:column;}.scaffold-body{position:relative;display:flex;width:100%;min-width:0;flex:1 1 auto;}.scaffold.is-boxed>.scaffold-body{max-width:72rem;margin-inline:auto;}.scaffold-main{position:relative;display:flex;min-width:0;flex:1 1 auto;flex-direction:column;}.scaffold-start,.scaffold-end{position:relative;flex:0 0 auto;}.scaffold-content{position:sticky;top:0;max-height:100vh;overflow:auto;}.scaffold-start .scaffold-content,.scaffold-end .scaffold-content{width:max-content;max-width:100vw;}");
    css.push_str(".navmenu{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;gap:0.25rem;position:relative;}.navmenu-item{display:inline-flex;align-items:center;gap:0.5rem;border:1px solid transparent;border-radius:var(--dowe-radiusBox);background:transparent;color:inherit;text-decoration:none;white-space:nowrap;cursor:pointer;user-select:none;outline:0;transition:background-color 180ms ease,color 180ms ease,border-color 180ms ease;font:inherit;}.navmenu-label{display:inline-flex;flex-direction:column;transition:font-weight 180ms ease;}.navmenu-label::after{content:attr(data-text);height:0;overflow:hidden;visibility:hidden;font-weight:700;user-select:none;pointer-events:none;}.navmenu-item:hover .navmenu-label,.navmenu-item.is-active .navmenu-label{font-weight:700;}.navmenu-icon,.navmenu-submenu-icon{display:flex;flex:0 0 auto;align-items:center;justify-content:center;}.navmenu-arrow{display:inline-flex;align-items:center;justify-content:center;transition:transform 180ms ease;}.navmenu-item.is-open .navmenu-arrow{transform:rotate(180deg);}.navmenu-popover{position:fixed;z-index:9999;display:flex;max-height:80vh;min-width:12rem;flex-direction:column;gap:0.5rem;overflow-y:auto;padding:0.5rem;border-radius:var(--dowe-radiusBox);background:var(--dowe-background);color:var(--dowe-onBackground);box-shadow:0 12px 32px rgba(15,23,42,0.18);opacity:0;visibility:hidden;pointer-events:none;transform:translateY(-0.25rem) scale(0.98);transform-origin:top center;transition:opacity 180ms ease,transform 180ms ease,visibility 180ms ease;}.navmenu-popover.is-above{transform:translateY(0.25rem) scale(0.98);transform-origin:bottom center;}.navmenu-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);}.navmenu-popover.is-megamenu{min-width:min(37.5rem,calc(100vw - 1rem));max-width:min(56rem,calc(100vw - 1rem));}.navmenu-popover-content{display:flex;flex-direction:column;gap:0.25rem;}.navmenu-submenu-item{display:flex;width:100%;align-items:center;gap:0.5rem;border:0;border-radius:var(--dowe-radiusUi);background:transparent;color:inherit;padding:0.5rem 0.75rem;text-align:left;text-decoration:none;cursor:pointer;font:inherit;transition:background-color 180ms ease,color 180ms ease;}.navmenu-submenu-item:hover,.navmenu-submenu-item.is-active{background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);}.navmenu-submenu-content{display:flex;min-width:0;flex:1 1 auto;flex-direction:column;}.navmenu-submenu-label{font-size:0.875rem;font-weight:600;line-height:1.25rem;}.navmenu-submenu-description{font-size:0.75rem;line-height:1rem;opacity:0.72;}.navmenu-sm{gap:0.125rem;}.navmenu-sm .navmenu-item{gap:0.375rem;padding:0.375rem 0.75rem;font-size:0.75rem;}.navmenu-sm .navmenu-icon,.navmenu-sm .navmenu-submenu-icon{width:1rem;height:1rem;}.navmenu-sm .navmenu-arrow{width:0.75rem;height:0.75rem;}.navmenu-md .navmenu-item{gap:0.5rem;padding:0.5rem 1rem;font-size:0.875rem;}.navmenu-md .navmenu-icon,.navmenu-md .navmenu-submenu-icon{width:1.25rem;height:1.25rem;}.navmenu-md .navmenu-arrow{width:1rem;height:1rem;}.navmenu-lg{gap:0.375rem;}.navmenu-lg .navmenu-item{gap:0.75rem;padding:0.75rem 1.5rem;font-size:1.125rem;}.navmenu-lg .navmenu-icon,.navmenu-lg .navmenu-submenu-icon{width:1.75rem;height:1.75rem;}.navmenu-lg .navmenu-arrow{width:1.5rem;height:1.5rem;}");
    css.push_str(".sidenav{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;gap:0.125rem;width:max-content;max-width:100%;}.sidenav.is-wide{width:100%;}.sidenav-entry,.sidenav-header{display:flex;align-items:center;gap:0.5rem;min-width:0;border:1px solid transparent;border-radius:var(--dowe-radiusUi);background:transparent;color:inherit;text-align:left;text-decoration:none;cursor:pointer;transition:background-color 160ms ease,color 160ms ease,border-color 160ms ease;}.sidenav.is-wide .sidenav-entry,.sidenav.is-wide .sidenav-header{width:100%;}.sidenav-header{margin-top:0.75rem;font-weight:700;}.sidenav-header:first-child{margin-top:0;}.sidenav-copy{display:flex;min-width:0;flex:1;flex-direction:column;}.sidenav-label,.sidenav-description{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}.sidenav-description{font-size:0.8em;opacity:0.72;}.sidenav-status{flex:0 0 auto;border-radius:999px;padding:0.125rem 0.5rem;background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);font-size:0.75em;font-weight:600;}.sidenav-divider{height:1px;margin:0.5rem 0;background:var(--dowe-muted);}.sidenav-icon{display:flex;flex:0 0 auto;align-items:center;justify-content:center;}.sidenav-submenu{display:flex;flex-direction:column;}.sidenav-submenu>summary{list-style:none;}.sidenav-submenu>summary::-webkit-details-marker{display:none;}.sidenav-submenu-content{display:flex;flex-direction:column;gap:0.125rem;max-height:0;overflow:hidden;margin-left:1rem;padding-left:0.5rem;border-left:1px solid var(--dowe-muted);opacity:0;transform:translateY(-0.25rem);transition:max-height 180ms ease,opacity 160ms ease,transform 160ms ease;}.sidenav-submenu.is-open>.sidenav-submenu-content{max-height:40rem;opacity:1;transform:translateY(0);}.sidenav-chevron{flex:0 0 auto;margin-left:auto;font-size:1.25em;line-height:1;transition:transform 160ms ease;}.sidenav-submenu.is-open>.sidenav-trigger .sidenav-chevron{transform:rotate(90deg);}.sidenav-sm .sidenav-entry,.sidenav-sm .sidenav-header{gap:0.375rem;padding:0.375rem 0.5rem;font-size:0.75rem;}.sidenav-md .sidenav-entry,.sidenav-md .sidenav-header{gap:0.5rem;padding:0.5rem 0.75rem;font-size:0.875rem;}.sidenav-lg .sidenav-entry,.sidenav-lg .sidenav-header{gap:0.75rem;padding:0.75rem 1rem;font-size:1rem;}");
    css.push_str(".sidebar{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;gap:0.125rem;width:max-content;max-width:100%;}.sidebar.is-wide{width:100%;}.sidebar-entry,.sidebar-header{display:flex;align-items:center;gap:0.5rem;min-width:0;border:1px solid transparent;border-radius:var(--dowe-radiusUi);background:transparent;color:inherit;text-align:left;text-decoration:none;cursor:pointer;transition:background-color 160ms ease,color 160ms ease,border-color 160ms ease;}.sidebar.is-wide .sidebar-entry,.sidebar.is-wide .sidebar-header{width:100%;}.sidebar-header{margin-top:0.75rem;font-weight:700;}.sidebar-header:first-child{margin-top:0;}.sidebar-copy{display:flex;min-width:0;flex:1;flex-direction:column;}.sidebar-label,.sidebar-description{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}.sidebar-description{font-size:0.8em;opacity:0.72;}.sidebar-status{flex:0 0 auto;border-radius:999px;padding:0.125rem 0.5rem;background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);font-size:0.75em;font-weight:600;}.sidebar-divider{height:1px;margin:0.5rem 0;background:var(--dowe-muted);}.sidebar-icon{display:flex;flex:0 0 auto;align-items:center;justify-content:center;}.sidebar-submenu{display:flex;flex-direction:column;}.sidebar-submenu>summary{list-style:none;}.sidebar-submenu>summary::-webkit-details-marker{display:none;}.sidebar-submenu-content{display:flex;flex-direction:column;gap:0.125rem;max-height:0;overflow:hidden;margin-left:1rem;padding-left:0.5rem;border-left:1px solid var(--dowe-muted);opacity:0;transform:translateY(-0.25rem);transition:max-height 180ms ease,opacity 160ms ease,transform 160ms ease;}.sidebar-submenu.is-open>.sidebar-submenu-content{max-height:40rem;opacity:1;transform:translateY(0);}.sidebar-chevron{flex:0 0 auto;margin-left:auto;font-size:1.25em;line-height:1;transition:transform 160ms ease;}.sidebar-submenu.is-open>.sidebar-trigger .sidebar-chevron{transform:rotate(90deg);}.sidebar-sm .sidebar-entry,.sidebar-sm .sidebar-header{gap:0.375rem;padding:0.375rem 0.5rem;font-size:0.75rem;}.sidebar-md .sidebar-entry,.sidebar-md .sidebar-header{gap:0.5rem;padding:0.5rem 0.75rem;font-size:0.875rem;}.sidebar-lg .sidebar-entry,.sidebar-lg .sidebar-header{gap:0.75rem;padding:0.75rem 1rem;font-size:1rem;}");
    css.push_str(".tabs{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));position:relative;gap:0.5rem;}.tabs.is-top{flex-direction:column;}.tabs.is-bottom{flex-direction:column-reverse;}.tabs.is-start{flex-direction:row;align-items:flex-start;}.tabs.is-end{flex-direction:row-reverse;align-items:flex-start;}.tabs-list{display:flex;width:auto;max-width:100%;overflow-x:auto;gap:0.5rem;padding:0.25rem;position:relative;user-select:none;scrollbar-width:none;}.tabs-list::-webkit-scrollbar{display:none;}.tabs.is-start .tabs-list,.tabs.is-end .tabs-list{flex-direction:column;overflow-x:visible;overflow-y:auto;max-height:100%;}.tab{display:inline-flex;align-items:center;white-space:nowrap;cursor:pointer;border:0;border-color:transparent;background:transparent;color:inherit;padding:0.25rem 1rem;position:relative;transition:color 160ms ease,background-color 160ms ease,border-color 160ms ease;font:inherit;}.tabs-label{white-space:nowrap;}.tabs-wrapper{display:flex;flex:1;min-width:0;overflow:hidden;position:relative;}.tabs-content{position:relative;width:100%;}");
    css.push_str(".drawer-panel{--dowe-component-display:flex;position:fixed;inset:0;z-index:50;display:var(--dowe-show,var(--dowe-component-display));}.drawer-overlay{position:absolute;inset:0;border:0;background:rgba(15,23,42,0.48);cursor:pointer;}.drawer{position:absolute;display:flex;max-width:100vw;max-height:100vh;flex-direction:column;overflow:auto;transition:transform 300ms ease-in-out;}.drawer.is-start{inset-block:0;inset-inline-start:0;width:min(20rem,100vw);border-start-start-radius:0;border-end-start-radius:0;transform:translateX(-100%);}.drawer.is-end{inset-block:0;inset-inline-end:0;width:min(20rem,100vw);border-start-end-radius:0;border-end-end-radius:0;transform:translateX(100%);}.drawer.is-top{inset-inline:0;top:0;max-height:min(20rem,100vh);border-start-start-radius:0;border-start-end-radius:0;transform:translateY(-100%);}.drawer.is-bottom{inset-inline:0;bottom:0;max-height:min(20rem,100vh);border-end-start-radius:0;border-end-end-radius:0;transform:translateY(100%);}.drawer.is-active{transform:translate(0,0);}.drawer-close{position:absolute;top:0.5rem;right:0.5rem;display:inline-flex;width:1.75rem;height:1.75rem;align-items:center;justify-content:center;border:0;border-radius:999px;background:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);font:inherit;font-size:1.25rem;line-height:1;cursor:pointer;}");
    css.push_str("@keyframes dowe-fade-in{from{opacity:0;}to{opacity:1;}}@keyframes dowe-slide-up{from{opacity:0;transform:translateY(1rem);}to{opacity:1;transform:translateY(0);}}@keyframes dowe-slide-down{from{opacity:0;transform:translateY(-1rem);}to{opacity:1;transform:translateY(0);}}@keyframes dowe-slide-left{from{opacity:0;transform:translateX(1rem);}to{opacity:1;transform:translateX(0);}}@keyframes dowe-slide-right{from{opacity:0;transform:translateX(-1rem);}to{opacity:1;transform:translateX(0);}}@keyframes dowe-scale-in{from{opacity:0;transform:scale(0.96);}to{opacity:1;transform:scale(1);}}@media (prefers-reduced-motion:reduce){*,::before,::after{animation-duration:1ms!important;transition-duration:1ms!important;scroll-behavior:auto!important;}}");

    css
}

fn append_theme_variables(css: &mut String, theme: &DesignTheme) {
    for token in ColorToken::all() {
        css.push_str(&format!(
            "--dowe-{}:{};",
            token.as_str(),
            theme.color_value(*token)
        ));
    }
    css.push_str(&format!(
        "--dowe-radius:{}px;--dowe-radiusBox:{}px;--dowe-radiusUi:{}px;",
        theme.radii.radius, theme.radii.radius_box, theme.radii.radius_ui
    ));
}

fn append_visibility_css(css: &mut String) {
    for value in [false, true] {
        css.push_str(&format!(
            ".show-{value}{{display:{};}}",
            visibility_display(value)
        ));
    }
    for breakpoint in [
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ] {
        css.push_str(&format!(
            "@media (min-width:{}px){{",
            breakpoint.min_width()
        ));
        for value in [false, true] {
            css.push_str(&format!(
                ".{}\\:show-{value}{{display:{};}}",
                breakpoint.as_str(),
                visibility_display(value)
            ));
        }
        css.push('}');
    }
}

fn visibility_display(value: bool) -> &'static str {
    if value {
        "var(--dowe-component-display,revert)"
    } else {
        "none"
    }
}

fn font_stack(value: FontFamily) -> &'static str {
    value.catalog_entry().web_stack
}

fn navigation_action_json(action: &ViewNavigationAction) -> String {
    match &action.action {
        NavigationAction::Internal {
            path,
            fragment,
            operation,
        } => format!(
            r#"{{"id":"{}","kind":"internal","operation":"{}","path":"{}","fragment":{}}}"#,
            escape_json(&action.id),
            operation.as_str(),
            escape_json(path),
            json_optional_string(fragment.as_deref())
        ),
        NavigationAction::Section {
            fragment,
            operation,
        } => format!(
            r#"{{"id":"{}","kind":"section","operation":"{}","fragment":"{}"}}"#,
            escape_json(&action.id),
            operation.as_str(),
            escape_json(fragment)
        ),
        NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        } => format!(
            r#"{{"id":"{}","kind":"external","operation":"external","url":"{}","webTarget":"{}","nativeExternalMode":"{}"}}"#,
            escape_json(&action.id),
            escape_json(url),
            web_target.as_str(),
            native_external_mode.as_str()
        ),
        NavigationAction::Back => format!(
            r#"{{"id":"{}","kind":"history","operation":"back"}}"#,
            escape_json(&action.id)
        ),
    }
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

pub fn router_js(web: &WebOutput) -> String {
    let routes = web
        .pages
        .iter()
        .map(|page| {
            let layout_chunks = page
                .layout_chunk_ids
                .iter()
                .map(|id| format!(r#""{id}""#))
                .collect::<Vec<_>>()
                .join(",");
            let js_chunks = page
                .js_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let css_chunks = page
                .css_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#""{}":{{id:"{}",path:"{}",layoutChunks:[{layout_chunks}],pageChunk:"{}",jsChunks:[{js_chunks}],cssChunks:[{css_chunks}]}}"#,
                escape_js(&page.route_path),
                escape_js(&page.id),
                escape_js(&page.route_path),
                escape_js(&page.page_chunk_id)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let initial_path = web
        .pages
        .first()
        .map(|page| escape_js(&page.route_path))
        .unwrap_or_else(|| "/".to_string());
    let locale_chunks = web
        .translation_chunks
        .iter()
        .map(|chunk| {
            format!(
                r#""{}":"{}""#,
                escape_js(&chunk.locale),
                escape_js(
                    chunk
                        .relative_path
                        .strip_prefix("web")
                        .unwrap_or(&chunk.relative_path)
                        .to_string_lossy()
                        .trim_start_matches('/')
                )
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let default_locale = web
        .default_locale
        .as_deref()
        .map(escape_js)
        .unwrap_or_default();
    minify_js(&format!(
        r##"
const routes={{{routes}}};
const initialPath="{initial_path}";
const localeChunks={{{locale_chunks}}};
const defaultLocale="{default_locale}";
const appElement=document.getElementById("dowe-app");
const staticMode=location.protocol==="file:";
function asset(path){{const script=document.querySelector('script[src$="router.js"]');return new URL(path,script?script.src:document.baseURI).href;}}
function normalizePath(path){{const normalized=(path||"/").replace(/\/$/,"");return normalized||"/";}}
function decodeFragment(hash){{return hash?decodeURIComponent(hash.slice(1)):"";}}
function splitStaticHash(hash){{if(!hash.startsWith("#/"))return null;const value=hash.slice(1);const fragmentIndex=value.indexOf("#");const path=fragmentIndex===-1?value:value.slice(0,fragmentIndex);const fragment=fragmentIndex===-1?"":decodeURIComponent(value.slice(fragmentIndex+1));return{{path:normalizePath(path),fragment,href:hash}};}}
function locationDestination(){{const staticDestination=splitStaticHash(location.hash);if(staticDestination)return staticDestination;return{{path:normalizePath(location.pathname),fragment:decodeFragment(location.hash),href:location.pathname+location.hash}};}}
const startupDestination=locationDestination();
let currentRoute=routes[startupDestination.path]||routes[appElement?.dataset.doweRoute]||routes[initialPath]||null;
let currentFragment=routes[startupDestination.path]?startupDestination.fragment:decodeFragment(location.hash);
let envPromise=null;
function splitDestination(value){{if(value.startsWith("#/"))return splitStaticHash(value);if(value.startsWith("#"))return{{path:currentRoute?currentRoute.path:initialPath,fragment:decodeFragment(value),href:value}};const url=new URL(value,location.href);return{{path:normalizePath(url.pathname),fragment:decodeFragment(url.hash),href:url.pathname+url.hash}};}}
function loadCss(path){{const href=asset(path);if(document.querySelector(`link[data-dowe-css="${{path}}"]`))return;const link=document.createElement("link");link.rel="stylesheet";link.href=href;link.dataset.doweCss=path;document.head.appendChild(link);}}
async function loadChunk(path){{return import(asset(path));}}
let translationsPromise=null;
function localeCandidates(){{const values=Array.isArray(navigator.languages)&&navigator.languages.length?navigator.languages:[navigator.language||""];return values.flatMap(value=>{{const locale=String(value).toLowerCase();const primary=locale.split("-")[0];return locale===primary?[locale]:[locale,primary];}});}}
function resolveLocale(){{for(const locale of localeCandidates())if(localeChunks[locale])return locale;return defaultLocale;}}
async function loadTranslations(){{if(!translationsPromise){{const locale=resolveLocale();translationsPromise=locale&&localeChunks[locale]?loadChunk(localeChunks[locale]).then(module=>module.translations||{{}}).catch(()=>({{}})):Promise.resolve({{}});}}return translationsPromise;}}
async function hydrateTranslations(root){{const translations=await loadTranslations();for(const element of root.querySelectorAll("[data-dowe-i18n]")){{const value=translations[element.dataset.doweI18n];if(value!=null)element.textContent=String(value);}}}}
async function loadEnv(){{if(!envPromise)envPromise=fetch(asset("env.json"),{{cache:"no-store"}}).then(response=>response.ok?response.json():{{}}).catch(()=>({{}}));return envPromise;}}
function wrapPage(route,html){{return `<div data-dowe-boundary="page:${{route.pageChunk}}">${{html}}</div>`;}}
function wrapLayout(route,html){{const id=route.layoutChunks[0];return id?`<div data-dowe-boundary="layout:${{id}}">${{html}}</div>`:html;}}
let activeView=null;
function cloneValue(value){{return value&&typeof value==="object"?JSON.parse(JSON.stringify(value)):value;}}
function readPath(state,path,scope){{if(!path)return undefined;const parts=path.split(".");let current;if(scope&&Object.prototype.hasOwnProperty.call(scope,parts[0]))current=scope[parts.shift()];else current=state[parts.shift()];for(const part of parts){{if(current==null)return undefined;current=current[part];}}return current;}}
function writePath(state,path,value){{const parts=path.split(".");let current=state;for(let i=0;i<parts.length-1;i++){{const part=parts[i];if(!current[part]||typeof current[part]!=="object")current[part]={{}};current=current[part];}}current[parts[parts.length-1]]=value;}}
function scopeFor(element){{const row=element&&element.closest?element.closest("[data-dowe-each-row]"):null;return row&&row.__doweScope?row.__doweScope:null;}}
function fillPath(path,state,body,scope){{return path.replace(/:([A-Za-z_][A-Za-z0-9_]*)/g,(_,name)=>{{const fromBody=body&&body[name]!=null?body[name]:undefined;const fromScope=readPath(state,name,scope);const value=fromBody!=null?fromBody:fromScope;return encodeURIComponent(value==null?"":String(value));}});}}
function requestUrl(base,path){{if(!base)return path;const cleanBase=String(base).replace(/\/+$/,"");const cleanPath=String(path).replace(/^\/+/,"");return `${{cleanBase}}/${{cleanPath}}`;}}
function setAlert(state,name,type,message){{if(!name)return;state[name]={{type,message,visible:true}};}}
function selectHost(control){{return control?control.closest(".select"):null;}}
function selectPopover(control){{if(!control)return null;if(control.__dowePopover)return control.__dowePopover;const host=selectHost(control);return host?host.querySelector("[data-dowe-select-popover]"):null;}}
function mountSelectPopover(control){{const popover=selectPopover(control);if(!popover)return null;const host=selectHost(control);popover.__doweControl=control;popover.__doweHost=host;control.__dowePopover=popover;if(popover.parentElement!==document.body)document.body.appendChild(popover);return popover;}}
function unmountSelectPopover(popover){{const host=popover&&popover.__doweHost;if(!popover||!host||popover.classList.contains("is-active"))return;if(popover.parentElement!==host)host.appendChild(popover);popover.style.left="";popover.style.top="";popover.style.width="";popover.style.fontFamily="";popover.style.fontSize="";}}
function selectOptions(control){{const popover=selectPopover(control);return popover?Array.from(popover.querySelectorAll("[data-dowe-option-value]")):[];}}
function closeSelect(control){{const popover=selectPopover(control);if(popover){{popover.classList.remove("is-active","is-above");setTimeout(()=>unmountSelectPopover(popover),180);}}if(control){{control.classList.remove("is-open");control.setAttribute("aria-expanded","false");}}}}
function closeSelects(except=null){{for(const control of document.querySelectorAll("[data-dowe-select].is-open"))if(control!==except)closeSelect(control);}}
function positionSelect(control){{const popover=mountSelectPopover(control);if(!popover)return;const rect=control.getBoundingClientRect();const style=getComputedStyle(control);popover.style.left=`${{rect.left}}px`;popover.style.width=`${{rect.width}}px`;popover.style.fontFamily=style.fontFamily;popover.style.fontSize=style.fontSize;popover.classList.remove("is-above");const height=popover.getBoundingClientRect().height;const bottom=window.innerHeight-rect.bottom;const top=rect.top;const above=bottom<Math.min(height,224)&&top>bottom;popover.classList.toggle("is-above",above);popover.style.top=`${{above?Math.max(8,rect.top-height-8):rect.bottom+4}}px`;requestAnimationFrame(()=>popover.classList.add("is-active"));}}
function openSelect(control){{closeSelects(control);control.classList.add("is-open");control.setAttribute("aria-expanded","true");positionSelect(control);}}
function renderSelect(control,state,scope){{const bound=control.dataset.doweBind;const raw=bound&&state?readPath(state,bound,scope):control.dataset.doweValue;const value=raw==null?"":String(raw);control.dataset.doweValue=value;const placeholder=control.dataset.dowePlaceholder||"Select an option";let label="";for(const option of selectOptions(control)){{const selected=option.dataset.doweOptionValue===value;option.classList.toggle("is-selected",selected);option.setAttribute("aria-selected",selected?"true":"false");if(selected)label=option.dataset.doweOptionLabel||option.textContent||"";}}const text=control.querySelector(".select-value");if(text)text.textContent=label||placeholder;control.classList.toggle("has-value",!!label);}}
function renderSelects(root,state,scope){{const scoped=!!scope;for(const control of root.querySelectorAll("[data-dowe-select]")){{if(!scoped&&control.closest("[data-dowe-each-row]"))continue;renderSelect(control,state,scope);}}}}
function setDrawerOpen(drawer,open){{if(drawer.__doweCloseTimer){{clearTimeout(drawer.__doweCloseTimer);drawer.__doweCloseTimer=null;}}const surface=drawer.querySelector(".drawer");if(open&&drawer.dataset.doweShowResolved!=="false"){{drawer.hidden=false;requestAnimationFrame(()=>surface?.classList.add("is-active"));return;}}surface?.classList.remove("is-active");drawer.__doweCloseTimer=setTimeout(()=>{{if(!surface?.classList.contains("is-active"))drawer.hidden=true;}},300);}}
function renderDrawers(root,state,scope){{const scoped=!!scope;for(const drawer of root.querySelectorAll("[data-dowe-drawer]")){{if(!scoped&&drawer.closest("[data-dowe-each-row]"))continue;setDrawerOpen(drawer,!!readPath(state,drawer.dataset.doweDrawerOpen,scope));}}}}
function closeDrawer(drawer){{if(!drawer||!activeView)return;writePath(activeView.state,drawer.dataset.doweDrawerOpen,false);renderReactive(activeView);}}
function closeDrawers(){{for(const drawer of document.querySelectorAll("[data-dowe-drawer]"))closeDrawer(drawer);}}
function setModalOpen(modal,open){{const surface=modal.querySelector(".modal,.command");if(open&&modal.dataset.doweShowResolved!=="false"){{modal.hidden=false;requestAnimationFrame(()=>surface?.classList.add("is-active"));return;}}surface?.classList.remove("is-active");modal.hidden=true;}}
function renderModals(root,state,scope){{const scoped=!!scope;for(const modal of root.querySelectorAll("[data-dowe-modal]")){{if(!scoped&&modal.closest("[data-dowe-each-row]"))continue;setModalOpen(modal,!!readPath(state,modal.dataset.doweModalOpen,scope));}}for(const command of root.querySelectorAll("[data-dowe-command][data-dowe-command-open]")){{if(!scoped&&command.closest("[data-dowe-each-row]"))continue;setModalOpen(command,!!readPath(state,command.dataset.doweCommandOpen,scope));}}}}
function closeModal(modal){{if(!modal||!activeView)return;const path=modal.dataset.doweModalOpen||modal.dataset.doweCommandOpen;if(path)writePath(activeView.state,path,false);const action=modal.dataset.doweModalOnClose;if(action)runAction(action,scopeFor(modal));renderReactive(activeView);}}
function closeModals(){{for(const modal of document.querySelectorAll("[data-dowe-modal],[data-dowe-command]"))closeModal(modal);}}
function closeDropdowns(except=null){{for(const root of document.querySelectorAll("[data-dowe-dropdown].is-open"))if(root!==except){{root.classList.remove("is-open");const pop=root.querySelector(".dropdown-popover");if(pop){{pop.classList.remove("is-active");pop.hidden=true;}}}}}}
function positionDropdown(root){{const trigger=root.querySelector("[data-dowe-dropdown-trigger]");const pop=root.querySelector(".dropdown-popover");if(!trigger||!pop)return;pop.hidden=false;const rect=trigger.getBoundingClientRect();const width=pop.getBoundingClientRect().width;const right=rect.left+width>window.innerWidth-8;pop.style.left=`${{Math.max(8,right?rect.right-width:rect.left)}}px`;pop.style.top=`${{Math.min(window.innerHeight-8,rect.bottom+8)}}px`;requestAnimationFrame(()=>pop.classList.add("is-active"));}}
function openDropdown(root){{closeDropdowns(root);root.classList.add("is-open");positionDropdown(root);}}
function tooltipPosition(root){{const pop=root.querySelector(".tooltip-popover");if(!pop)return;const rect=root.getBoundingClientRect();pop.classList.add("is-active");const pr=pop.getBoundingClientRect();let top=rect.top-pr.height-8;let left=rect.left+rect.width/2-pr.width/2;if(pop.classList.contains("position-bottom"))top=rect.bottom+8;if(pop.classList.contains("position-start")){{top=rect.top+rect.height/2-pr.height/2;left=rect.left-pr.width-8;}}if(pop.classList.contains("position-end")){{top=rect.top+rect.height/2-pr.height/2;left=rect.right+8;}}pop.style.top=`${{Math.max(8,Math.min(top,window.innerHeight-pr.height-8))}}px`;pop.style.left=`${{Math.max(8,Math.min(left,window.innerWidth-pr.width-8))}}px`;}}
function closeTooltips(){{for(const pop of document.querySelectorAll(".tooltip-popover.is-active"))pop.classList.remove("is-active");}}
function renderToasts(root,state,scope){{const scoped=!!scope;for(const toast of root.querySelectorAll("[data-dowe-toast]")){{if(!scoped&&toast.closest("[data-dowe-each-row]"))continue;const source=toast.dataset.doweToastSource;if(!source)continue;const value=readPath(state,source,scope)||{{}};toast.hidden=!(value.visible&&toast.dataset.doweShowResolved!=="false");const title=toast.querySelector(".toast-title");const desc=toast.querySelector(".toast-description");if(title){{title.textContent=value.title||"";title.hidden=!value.title;}}if(desc)desc.textContent=value.message||"";}}}}
function closeToast(toast){{if(!toast)return;const source=toast.dataset.doweToastSource;if(activeView&&source){{const value=readPath(activeView.state,source)||{{}};value.visible=false;writePath(activeView.state,source,value);renderReactive(activeView);}}else toast.hidden=true;}}
function openCommand(command){{if(activeView&&command.dataset.doweCommandOpen){{writePath(activeView.state,command.dataset.doweCommandOpen,true);renderReactive(activeView);}}else setModalOpen(command,true);const input=command.querySelector("[data-dowe-command-input]");setTimeout(()=>input?.focus(),0);}}
function closeCommand(command){{if(activeView&&command?.dataset.doweCommandOpen){{writePath(activeView.state,command.dataset.doweCommandOpen,false);renderReactive(activeView);}}else if(command)setModalOpen(command,false);}}
function filterCommand(command){{const input=command.querySelector("[data-dowe-command-input]");const query=(input?.value||"").toLowerCase();let any=false;for(const item of command.querySelectorAll(".command-item")){{const label=(item.textContent||"").toLowerCase();const show=!query||label.includes(query);item.hidden=!show;any=any||show;}}const empty=command.querySelector(".command-empty");if(empty)empty.hidden=any;}}
function setActiveTab(root,id){{if(!root||!id)return;for(const tab of root.querySelectorAll("[data-dowe-tab]")){{const active=tab.dataset.doweTab===id;tab.classList.toggle("on-active",active);tab.setAttribute("aria-selected",active?"true":"false");tab.tabIndex=active?0:-1;}}for(const panel of root.querySelectorAll("[data-dowe-tab-panel]")){{const active=panel.dataset.doweTabPanel===id;panel.classList.toggle("on-active",active);panel.hidden=!active;}}}}
function moveActiveTab(tab,step){{const list=tab.closest("[role='tablist']");if(!list)return;const tabs=Array.from(list.querySelectorAll("[data-dowe-tab]"));const index=tabs.indexOf(tab);if(index<0||!tabs.length)return;const next=tabs[(index+step+tabs.length)%tabs.length];if(!next)return;setActiveTab(next.closest("[data-dowe-tabs]"),next.dataset.doweTab);next.focus();}}
function edgeActiveTab(tab,end){{const list=tab.closest("[role='tablist']");if(!list)return;const tabs=Array.from(list.querySelectorAll("[data-dowe-tab]"));const next=end?tabs[tabs.length-1]:tabs[0];if(!next)return;setActiveTab(next.closest("[data-dowe-tabs]"),next.dataset.doweTab);next.focus();}}
function audioTime(value){{if(!Number.isFinite(value))return "0:00";const minutes=Math.floor(value/60);const seconds=String(Math.floor(value%60)).padStart(2,"0");return minutes+":"+seconds;}}
function updateAudio(root){{const audio=root.querySelector("[data-dowe-audio-el]");if(!audio)return;const duration=Number(audio.duration)||0;const current=Number(audio.currentTime)||0;const progress=duration?current/duration:0;const time=root.querySelector("[data-dowe-audio-time]");if(time)time.textContent=duration?audioTime(Math.max(0,duration-current)):"0:00";const waveform=root.querySelector("[data-dowe-audio-waveform]");if(waveform){{waveform.setAttribute("aria-valuenow",String(Math.round(progress*100)));}}const bars=Array.from(root.querySelectorAll(".media-bar"));bars.forEach((bar,index)=>bar.classList.toggle("active",bars.length?((index+0.5)/bars.length)<=progress:false));const icon=root.querySelector("[data-dowe-audio-icon]");if(icon)icon.textContent=audio.paused?"▶":"Ⅱ";const toggle=root.querySelector("[data-dowe-audio-toggle]");if(toggle)toggle.setAttribute("aria-label",audio.paused?"Play audio":"Pause audio");}}
function hydrateAudios(root){{for(const media of root.querySelectorAll("[data-dowe-audio]")){{if(media.__doweAudioHydrated)continue;media.__doweAudioHydrated=true;const audio=media.querySelector("[data-dowe-audio-el]");if(!audio)continue;audio.addEventListener("loadedmetadata",()=>updateAudio(media));audio.addEventListener("timeupdate",()=>updateAudio(media));audio.addEventListener("ended",()=>updateAudio(media));updateAudio(media);}}}}
function toggleAccordion(trigger){{const item=trigger.closest("[data-dowe-accordion-item]");const root=trigger.closest("[data-dowe-accordion]");if(!item||!root||trigger.disabled)return;const open=item.classList.contains("is-open");if(root.dataset.doweAccordionMultiple!=="true"){{for(const other of root.querySelectorAll("[data-dowe-accordion-item].is-open"))if(other!==item){{other.classList.remove("is-open");const button=other.querySelector("[data-dowe-accordion-trigger]");const content=other.querySelector("[data-dowe-accordion-content]");if(button){{button.classList.remove("is-open");button.setAttribute("aria-expanded","false");}}if(content)content.hidden=true;}}}}item.classList.toggle("is-open",!open);trigger.classList.toggle("is-open",!open);trigger.setAttribute("aria-expanded",open?"false":"true");const content=item.querySelector("[data-dowe-accordion-content]");if(content)content.hidden=open;}}
function renderCarousel(root){{const slides=Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"));if(!slides.length)return;let index=Number(root.dataset.doweCarouselIndex||0);index=Math.max(0,Math.min(index,slides.length-1));root.dataset.doweCarouselIndex=String(index);const track=root.querySelector("[data-dowe-carousel-track]");const vertical=root.dataset.doweCarouselOrientation==="vertical";const prefix=vertical?"translateY(":"translateX(";if(track)track.style.transform=prefix+(-index*100)+"%)";for(const indicator of root.querySelectorAll("[data-dowe-carousel-indicator]"))indicator.classList.toggle("is-active",Number(indicator.dataset.doweCarouselIndicator)===index);const counter=root.querySelector("[data-dowe-carousel-counter]");if(counter)counter.textContent=String(index+1)+" / "+String(slides.length);}}
function moveCarousel(root,step){{const slides=root?Array.from(root.querySelectorAll("[data-dowe-carousel-slide]")):[];if(!root||!slides.length)return;const loop=root.dataset.doweCarouselLoop==="true";let index=Number(root.dataset.doweCarouselIndex||0)+step;if(index<0)index=loop?slides.length-1:0;if(index>=slides.length)index=loop?0:slides.length-1;root.dataset.doweCarouselIndex=String(index);renderCarousel(root);}}
function hydrateCarousels(root){{for(const carousel of root.querySelectorAll("[data-dowe-carousel]")){{renderCarousel(carousel);if(carousel.dataset.doweCarouselAutoplay==="true"&&!carousel.__doweCarouselTimer){{const interval=Math.max(500,Number(carousel.dataset.doweCarouselInterval||3000));carousel.__doweCarouselTimer=setInterval(()=>{{if(!carousel.isConnected){{clearInterval(carousel.__doweCarouselTimer);carousel.__doweCarouselTimer=null;return;}}moveCarousel(carousel,1);}},interval);}}}}}}
function downloadImage(root){{const img=root?.querySelector("img");const src=img?.currentSrc||img?.src;if(!src)return;fetch(src).then(response=>response.blob()).then(blob=>{{const url=URL.createObjectURL(blob);const link=document.createElement("a");link.href=url;link.download=img.alt||"image";document.body.appendChild(link);link.click();link.remove();URL.revokeObjectURL(url);}}).catch(()=>{{const link=document.createElement("a");link.href=src;link.download=img.alt||"image";document.body.appendChild(link);link.click();link.remove();}});}}
function toggleImageFullscreen(root){{if(!root)return;if(!document.fullscreenElement&&root.requestFullscreen)root.requestFullscreen();else if(document.exitFullscreen)document.exitFullscreen();}}
function isCandle(value){{return value&&Number.isFinite(Number(value.open))&&Number.isFinite(Number(value.high))&&Number.isFinite(Number(value.low))&&Number.isFinite(Number(value.close))&&(typeof value.time==="string"||typeof value.time==="number")&&Number(value.high)>=Math.max(Number(value.open),Number(value.close))&&Number(value.low)<=Math.min(Number(value.open),Number(value.close));}}
function candleList(value){{return Array.isArray(value)?value.filter(isCandle):[];}}
function tokenColor(name){{return getComputedStyle(document.documentElement).getPropertyValue("--dowe-"+name).trim()||"currentColor";}}
function candleY(value,min,max,height,pad){{return pad+(max-value)/(max-min)*(height-pad*2);}}
function renderCandlestick(chart,state,scope){{const canvas=chart.querySelector("canvas");if(!canvas)return;const data=candleList(readPath(state,chart.dataset.doweCandlestickData,scope));chart.classList.toggle("has-data",data.length>0);const rect=chart.getBoundingClientRect();const width=Math.max(1,Math.floor(rect.width));const height=Math.max(1,Math.floor(rect.height));const ratio=window.devicePixelRatio||1;if(canvas.width!==Math.floor(width*ratio)||canvas.height!==Math.floor(height*ratio)){{canvas.width=Math.floor(width*ratio);canvas.height=Math.floor(height*ratio);}}canvas.style.width=width+"px";canvas.style.height=height+"px";const ctx=canvas.getContext("2d");if(!ctx)return;ctx.setTransform(ratio,0,0,ratio,0,0);ctx.clearRect(0,0,width,height);if(!data.length)return;const pad=Math.min(32,Math.max(16,height*0.12));let min=Math.min(...data.map(value=>Number(value.low)));let max=Math.max(...data.map(value=>Number(value.high)));if(min===max){{min-=1;max+=1;}}ctx.lineWidth=1;ctx.strokeStyle="currentColor";ctx.globalAlpha=0.14;for(let index=0;index<5;index++){{const y=pad+(height-pad*2)*(index/4);ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(width,y);ctx.stroke();}}ctx.globalAlpha=1;const plotWidth=Math.max(1,width);const step=plotWidth/data.length;const bodyWidth=Math.max(3,Math.min(18,step*0.56));const up=tokenColor(chart.dataset.doweCandlestickUp||"success");const down=tokenColor(chart.dataset.doweCandlestickDown||"danger");data.forEach((candle,index)=>{{const open=Number(candle.open);const high=Number(candle.high);const low=Number(candle.low);const close=Number(candle.close);const x=step*index+step/2;const color=close>=open?up:down;const highY=candleY(high,min,max,height,pad);const lowY=candleY(low,min,max,height,pad);const openY=candleY(open,min,max,height,pad);const closeY=candleY(close,min,max,height,pad);ctx.strokeStyle=color;ctx.fillStyle=color;ctx.beginPath();ctx.moveTo(x,highY);ctx.lineTo(x,lowY);ctx.stroke();const top=Math.min(openY,closeY);const bodyHeight=Math.max(1,Math.abs(closeY-openY));ctx.fillRect(x-bodyWidth/2,top,bodyWidth,bodyHeight);}});}}
function renderCandlesticks(root,state,scope){{const scoped=!!scope;for(const chart of root.querySelectorAll("[data-dowe-candlestick]")){{if(!scoped&&chart.closest("[data-dowe-each-row]"))continue;renderCandlestick(chart,state,scope);}}}}
function upsertCandles(current,payload,max){{const values=Array.isArray(payload)?payload:[payload];let output=Array.isArray(current)?current.slice():[];for(const value of values){{if(!isCandle(value))continue;const last=output[output.length-1];if(last&&String(last.time)===String(value.time))output[output.length-1]=value;else output.push(value);}}if(output.length>max)output=output.slice(output.length-max);return output;}}
function closeCandlestickStreams(view){{for(const stream of view?.streams||[])try{{stream.close();}}catch(error){{}}}}
function hydrateCandlesticks(view){{renderCandlesticks(view.root,view.state,null);for(const chart of view.root.querySelectorAll("[data-dowe-candlestick-stream]")){{const stream=chart.dataset.doweCandlestickStream;if(!stream||chart.__doweStreamSource===stream)continue;chart.__doweStreamSource=stream;const source=new EventSource(stream);source.onmessage=event=>{{try{{const payload=JSON.parse(event.data);const path=chart.dataset.doweCandlestickData;const max=Number(chart.dataset.doweCandlestickMax||240);writePath(view.state,path,upsertCandles(readPath(view.state,path),payload,max));renderReactive(view);}}catch(error){{}}}};source.onerror=()=>{{}};view.streams.push(source);}}}}
function tableCellValue(row,path){{let current=row;for(const part of String(path||"").split(".")){{if(!part)return "";if(current==null)return "";current=current[part];}}return current==null||typeof current==="object"?"":String(current);}}
function tableColumns(table){{return Array.from(table.querySelectorAll("[data-dowe-table-field]")).map(head=>({{field:head.dataset.doweTableField||"",align:head.dataset.doweTableAlign||"start"}}));}}
function renderTableEmpty(table,body,columns){{const row=document.createElement("tr");row.className="table-empty-row";const cell=document.createElement("td");cell.className="table-empty-cell";cell.colSpan=Math.max(1,columns.length);const state=document.createElement("div");state.className="empty-state";const content=document.createElement("div");content.className="empty-content";const title=document.createElement("h3");title.className="empty-title";title.textContent=table.dataset.doweTableEmptyTitle||"No data";const description=document.createElement("p");description.className="empty-description";description.textContent=table.dataset.doweTableEmptyDescription||"There are no records to display";content.append(title,description);state.appendChild(content);cell.appendChild(state);row.appendChild(cell);body.appendChild(row);}}
function renderTable(table,state,scope){{const body=table.querySelector(".table-body");if(!body)return;const columns=tableColumns(table);body.innerHTML="";const rows=readPath(state,table.dataset.doweTableData,scope);const values=Array.isArray(rows)?rows:[];if(!values.length){{renderTableEmpty(table,body,columns);return;}}for(const value of values){{const row=document.createElement("tr");for(const column of columns){{const cell=document.createElement("td");cell.style.textAlign=column.align==="end"?"end":column.align==="center"?"center":"start";cell.textContent=tableCellValue(value,column.field);row.appendChild(cell);}}body.appendChild(row);}}}}
function renderTables(root,state,scope){{const scoped=!!scope;for(const table of root.querySelectorAll("[data-dowe-table]")){{if(!scoped&&table.closest("[data-dowe-each-row]"))continue;renderTable(table,state,scope);}}}}
function renderDynamic(root,state,scope){{const scoped=!!scope;for(const element of root.querySelectorAll("[data-dowe-text]")){{if(!scoped&&element.closest("[data-dowe-each-row]"))continue;const value=readPath(state,element.dataset.doweText,scope);element.textContent=value==null?"":String(value);}}for(const input of root.querySelectorAll("[data-dowe-bind]:not([data-dowe-select])")){{if(!scoped&&input.closest("[data-dowe-each-row]"))continue;const value=readPath(state,input.dataset.doweBind,scope);if(input.type==="checkbox"){{input.checked=!!value;input.setAttribute("aria-checked",input.checked?"true":"false");}}else if(input.type==="radio"){{input.checked=String(input.value)===String(value==null?"":value);}}else if(document.activeElement!==input)input.value=value==null?"":String(value);const control=input.closest(".control");if(control)control.classList.toggle("has-value",value!=null&&String(value)!=="");}}renderSelects(root,state,scope);for(const element of root.querySelectorAll("[data-dowe-show]")){{if(!scoped&&element.closest("[data-dowe-each-row]"))continue;const visible=!!readPath(state,element.dataset.doweShow,scope);element.dataset.doweShowResolved=visible?"true":"false";element.hidden=!visible;}}renderDrawers(root,state,scope);renderModals(root,state,scope);renderToasts(root,state,scope);renderTables(root,state,scope);for(const alert of root.querySelectorAll("[data-dowe-alert]")){{const path=alert.dataset.doweAlertVisible;const visible=path?!!readPath(state,path,scope):true;const showVisible=alert.dataset.doweShowResolved!=="false";alert.hidden=!showVisible||!visible;}}}}
function renderEach(root,state){{for(const container of root.querySelectorAll("[data-dowe-each]")){{const template=container.querySelector(":scope>template");if(!template)continue;for(const row of Array.from(container.querySelectorAll(":scope>[data-dowe-each-row]")))row.remove();const values=readPath(state,container.dataset.doweEach)||[];const item=container.dataset.doweItem;values.forEach((value,index)=>{{const row=document.createElement("div");row.dataset.doweEachRow="";row.dataset.doweEachIndex=String(index);row.innerHTML=template.innerHTML;row.__doweScope={{[item]:value}};container.appendChild(row);renderDynamic(row,state,row.__doweScope);}});}}}}
function renderReactive(view){{renderEach(view.root,view.state);renderDynamic(view.root,view.state,null);renderCandlesticks(view.root,view.state,null);}}
let hlsPromise=null;
function isHlsSource(source){{try{{return new URL(source,location.href).pathname.toLowerCase().endsWith(".m3u8");}}catch(error){{return false;}}}}
function loadHlsRuntime(){{if(window.Hls)return Promise.resolve(window.Hls);if(!hlsPromise)hlsPromise=new Promise((resolve,reject)=>{{const script=document.createElement("script");script.src="https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js";script.async=true;script.onload=()=>resolve(window.Hls);script.onerror=reject;document.head.appendChild(script);}});return hlsPromise;}}
function hydrateVideo(video){{const source=video.dataset.doweVideoSource||video.getAttribute("src")||"";if(!source||video.__doweVideoSource===source)return;video.__doweVideoSource=source;if(!isHlsSource(source)||video.canPlayType("application/vnd.apple.mpegurl")){{video.src=source;return;}}loadHlsRuntime().then(Hls=>{{if(!Hls||!Hls.isSupported()){{video.src=source;return;}}const hls=new Hls();hls.loadSource(source);hls.attachMedia(video);video.__doweHls=hls;}}).catch(()=>{{video.src=source;}});}}
function hydrateVideos(root){{for(const video of root.querySelectorAll("[data-dowe-video]"))hydrateVideo(video);}}
function renderNavigationActive(root,path){{for(const entry of root.querySelectorAll("[data-dowe-sidenav-href],[data-dowe-sidebar-href],[data-dowe-navmenu-href]")){{const value=entry.getAttribute("data-dowe-sidenav-href")||entry.getAttribute("data-dowe-sidebar-href")||entry.getAttribute("data-dowe-navmenu-href")||"";entry.classList.toggle("is-active",normalizePath(value)===normalizePath(path));}}}}
function setNavTreeSubmenu(base,details,open){{if(!details)return;const trigger=details.querySelector("."+base+"-trigger");if(open){{details.open=true;details.classList.remove("is-closing");if(trigger)trigger.setAttribute("aria-expanded","true");requestAnimationFrame(()=>details.classList.add("is-open"));}}else{{details.classList.remove("is-open");details.classList.add("is-closing");if(trigger)trigger.setAttribute("aria-expanded","false");setTimeout(()=>{{if(!details.classList.contains("is-open"))details.open=false;details.classList.remove("is-closing");}},180);}}}}
function toggleNavTreeSubmenu(base,trigger){{const details=trigger?trigger.closest("[data-dowe-"+base+"-submenu]"):null;if(!details)return false;setNavTreeSubmenu(base,details,!(details.open&&details.classList.contains("is-open")));return true;}}
function hydrateNavTreeSubmenus(root,base){{for(const details of root.querySelectorAll("[data-dowe-"+base+"-submenu]")){{const trigger=details.querySelector("."+base+"-trigger");details.classList.toggle("is-open",details.open);details.classList.remove("is-closing");if(trigger)trigger.setAttribute("aria-expanded",details.open?"true":"false");}}}}
function closeNavMenus(except=null){{for(const root of document.querySelectorAll("[data-dowe-navmenu]")){{if(root===except)continue;for(const trigger of root.querySelectorAll("[data-dowe-navmenu-trigger]")){{trigger.classList.remove("is-open");trigger.setAttribute("aria-expanded","false");}}for(const popover of root.querySelectorAll("[data-dowe-navmenu-popover]")){{popover.classList.remove("is-active","is-above");popover.hidden=true;}}}}}}
function positionNavMenu(trigger,popover){{if(!trigger||!popover)return;popover.hidden=false;const rect=trigger.getBoundingClientRect();const width=popover.getBoundingClientRect().width;const height=popover.getBoundingClientRect().height;let left=rect.left;if(popover.classList.contains("is-megamenu"))left=rect.left+rect.width/2-width/2;left=Math.max(8,Math.min(left,window.innerWidth-width-8));const above=rect.bottom+height+8>window.innerHeight&&rect.top>height;popover.classList.toggle("is-above",above);popover.style.left=`${{left}}px`;popover.style.top=`${{above?Math.max(8,rect.top-height-8):rect.bottom+8}}px`;}}
function openNavMenu(trigger){{const root=trigger?trigger.closest("[data-dowe-navmenu]"):null;if(!root)return false;const index=trigger.getAttribute("data-dowe-navmenu-trigger");const popover=root.querySelector(`[data-dowe-navmenu-popover="${{index}}"]`);if(!popover)return false;const open=trigger.classList.contains("is-open");closeNavMenus(root);if(open)return true;trigger.classList.add("is-open");trigger.setAttribute("aria-expanded","true");positionNavMenu(trigger,popover);requestAnimationFrame(()=>popover.classList.add("is-active"));return true;}}
function positionOpenNavMenu(){{const trigger=document.querySelector("[data-dowe-navmenu-trigger].is-open");if(!trigger)return;const root=trigger.closest("[data-dowe-navmenu]");const index=trigger.getAttribute("data-dowe-navmenu-trigger");const popover=root?root.querySelector(`[data-dowe-navmenu-popover="${{index}}"]`):null;positionNavMenu(trigger,popover);}}
function reactiveRoot(route){{const boundary=route.layoutChunks[0]?`layout:${{route.layoutChunks[0]}}`:`page:${{route.pageChunk}}`;return document.querySelector(`[data-dowe-boundary="${{boundary}}"]`);}}
function hydrate(route,modules,preserveLayouts=false){{const root=reactiveRoot(route);if(!root)return;closeCandlestickStreams(activeView);const state={{}};const initial={{}};const actions={{}};const autoload=[];for(const module of modules||[]){{const layout=!!module.doweLayout;const definition=module.doweLayout||module.dowePage;if(!definition)continue;for(const signal of definition.signals||[]){{const preserve=preserveLayouts&&layout&&activeView&&Object.prototype.hasOwnProperty.call(activeView.state,signal.id);state[signal.id]=preserve?cloneValue(activeView.state[signal.id]):cloneValue(signal.initial);initial[signal.id]=cloneValue(signal.initial);}}for(const action of definition.actions||[]){{actions[action.id]=action;if(action.autoload&&!(preserveLayouts&&layout))autoload.push(action.id);}}}}activeView={{root,state,initial,actions,streams:[]}};renderReactive(activeView);hydrateTranslations(root);hydrateVideos(root);hydrateAudios(root);hydrateCarousels(root);hydrateCandlesticks(activeView);hydrateNavTreeSubmenus(root,"sidenav");hydrateNavTreeSubmenus(root,"sidebar");renderNavigationActive(root,route.path);for(const id of autoload)runAction(id,null);}}
async function runAction(id,scope){{const view=activeView;if(!view)return;const action=view.actions[id];if(!action)return;const name=action.name;if(action.kind==="assign"){{view.state[action.target]=cloneValue(readPath(view.state,action.source,scope));renderReactive(view);return;}}if(action.kind==="reset"){{view.state[action.target]=cloneValue(view.initial[action.target]);renderReactive(view);return;}}if(action.kind!=="request")return;try{{const body=action.body?cloneValue(readPath(view.state,action.body,scope)):undefined;const path=fillPath(action.path,view.state,body,scope);const env=action.baseEnv?await loadEnv():{{}};const url=requestUrl(action.baseEnv?env[action.baseEnv]:"",path);const options={{method:action.method,headers:{{}}}};if(body!==undefined&&action.method!=="GET"){{options.headers["content-type"]="application/json";options.body=JSON.stringify(body);}}const response=await fetch(url,options);const payload=await response.json().catch(()=>({{}}));if(!response.ok||payload.ok===false)throw new Error(payload.error&&payload.error.message?payload.error.message:`Request failed with status ${{response.status}}`);if(action.update)view.state[action.update]=cloneValue(payload.data!==undefined?payload.data:payload);if(action.reset)view.state[action.reset]=cloneValue(view.initial[action.reset]);setAlert(view.state,action.successAlert,"success",action.successMessage||"Request completed");window.dispatchEvent(new CustomEvent("dowe:request",{{detail:{{name,ok:true,payload}}}}));}}catch(error){{setAlert(view.state,action.errorAlert,"error",action.errorMessage||error.message||"Request failed");window.dispatchEvent(new CustomEvent("dowe:request",{{detail:{{name,ok:false,error:String(error.message||error)}}}}));}}renderReactive(view);}}
async function loadRouteModules(route){{const modules=[];for(const path of route.jsChunks)modules.push(await loadChunk(path));return modules;}}
async function renderFull(route){{for(const path of route.cssChunks)loadCss(path);const modules=await loadRouteModules(route);const page=modules[modules.length-1];let html=wrapPage(route,page.render());for(let i=modules.length-2;i>=0;i--)html=modules[i].render(html);return{{html:wrapLayout(route,html),modules}};}}
function scrollToFragment(fragment){{if(!fragment)return;requestAnimationFrame(()=>{{const target=document.getElementById(fragment);if(!target)return;const reduce=window.matchMedia&&window.matchMedia("(prefers-reduced-motion: reduce)").matches;target.scrollIntoView({{behavior:reduce?"auto":"smooth",block:"start"}});if(!target.hasAttribute("tabindex"))target.setAttribute("tabindex","-1");target.focus({{preventScroll:true}});}});}}
function historyHref(route,fragment){{if(staticMode)return `#${{route.path}}${{fragment?`#${{encodeURIComponent(fragment)}}`:""}}`;return route.path+(fragment?`#${{encodeURIComponent(fragment)}}`:"");}}
function updateHistory(route,fragment,replace,write){{if(!write)return;const href=historyHref(route,fragment);const state={{path:route.path,fragment}};try{{if(replace)history.replaceState(state,"",href);else history.pushState(state,"",href);}}catch(error){{location.hash=href;}}}}
async function navigate(value,options={{}}){{const destination=splitDestination(value);const route=routes[destination.path]||null;if(!route){{if(options.writeHistory!==false)location.href=value;return;}}closeSelects();closeDrawers();closeNavMenus();const app=document.getElementById("dowe-app");if(!app)return;const sameRoute=currentRoute&&currentRoute.path===route.path;if(sameRoute&&destination.fragment!==currentFragment){{currentFragment=destination.fragment;updateHistory(route,currentFragment,!!options.replace,options.writeHistory!==false);scrollToFragment(currentFragment);return;}}for(const css of route.cssChunks)loadCss(css);const preserveLayouts=!!(currentRoute&&currentRoute.layoutChunks.join("|")===route.layoutChunks.join("|"));let modules=null;if(preserveLayouts){{modules=await loadRouteModules(route);const page=modules[modules.length-1];const boundary=document.querySelector('[data-dowe-boundary^="page:"]');if(boundary)boundary.outerHTML=wrapPage(route,page.render());else{{const rendered=await renderFull(route);app.innerHTML=rendered.html;modules=rendered.modules;}}}}else{{const rendered=await renderFull(route);app.innerHTML=rendered.html;modules=rendered.modules;}}app.dataset.doweRoute=route.path;currentRoute=route;currentFragment=destination.fragment;hydrate(route,modules,preserveLayouts);updateHistory(route,currentFragment,!!options.replace,options.writeHistory!==false);scrollToFragment(currentFragment);}}
function goBack(){{if(history.length>1)history.back();else navigate(initialPath,{{replace:true}});}}
window.addEventListener("popstate",()=>navigate(locationDestination().href,{{replace:true,writeHistory:false}}));
document.addEventListener("input",event=>{{const target=event.target;if(!activeView||!target||!target.dataset||!target.dataset.doweBind)return;if(target.type==="radio"&&!target.checked)return;const value=target.type==="checkbox"?target.checked:target.value;writePath(activeView.state,target.dataset.doweBind,value);renderReactive(activeView);}});
document.addEventListener("click",event=>{{const target=event.target;if(!target||!target.closest)return;const audioToggle=target.closest("[data-dowe-audio-toggle]");if(audioToggle){{event.preventDefault();const root=audioToggle.closest("[data-dowe-audio]");const audio=root?.querySelector("[data-dowe-audio-el]");if(audio){{if(audio.paused)audio.play().catch(()=>{{}});else audio.pause();setTimeout(()=>updateAudio(root),0);}}return;}}const accordionTrigger=target.closest("[data-dowe-accordion-trigger]");if(accordionTrigger){{event.preventDefault();toggleAccordion(accordionTrigger);return;}}const carouselPrev=target.closest("[data-dowe-carousel-prev]");if(carouselPrev){{event.preventDefault();moveCarousel(carouselPrev.closest("[data-dowe-carousel]"),-1);return;}}const carouselNext=target.closest("[data-dowe-carousel-next]");if(carouselNext){{event.preventDefault();moveCarousel(carouselNext.closest("[data-dowe-carousel]"),1);return;}}const carouselIndicator=target.closest("[data-dowe-carousel-indicator]");if(carouselIndicator){{event.preventDefault();const root=carouselIndicator.closest("[data-dowe-carousel]");if(root){{root.dataset.doweCarouselIndex=carouselIndicator.dataset.doweCarouselIndicator||"0";renderCarousel(root);}}return;}}const imageDownload=target.closest("[data-dowe-image-download]");if(imageDownload){{event.preventDefault();downloadImage(imageDownload.closest("[data-dowe-image]"));return;}}const imageFullscreen=target.closest("[data-dowe-image-fullscreen]");if(imageFullscreen){{event.preventDefault();toggleImageFullscreen(imageFullscreen.closest("[data-dowe-image]"));return;}}const tab=target.closest("[data-dowe-tab]");if(tab){{event.preventDefault();setActiveTab(tab.closest("[data-dowe-tabs]"),tab.dataset.doweTab);return;}}const dropdownTrigger=target.closest("[data-dowe-dropdown-trigger]");if(dropdownTrigger){{event.preventDefault();const root=dropdownTrigger.closest("[data-dowe-dropdown]");if(root.classList.contains("is-open"))closeDropdowns();else openDropdown(root);return;}}if(target.closest(".dropdown-item"))closeDropdowns();if(!target.closest("[data-dowe-dropdown]"))closeDropdowns();const option=target.closest("[data-dowe-option-value]");if(option){{const popover=option.closest("[data-dowe-select-popover]");const control=popover?(popover.__doweControl||(popover.__doweHost?popover.__doweHost.querySelector("[data-dowe-select]"):null)):null;if(control){{event.preventDefault();const value=option.dataset.doweOptionValue||"";if(control.dataset.doweBind&&activeView){{writePath(activeView.state,control.dataset.doweBind,value);renderReactive(activeView);}}else{{control.dataset.doweValue=value;renderSelect(control,activeView?activeView.state:null,null);}}closeSelect(control);}}return;}}const control=target.closest("[data-dowe-select]");if(control){{event.preventDefault();if(control.classList.contains("is-open"))closeSelect(control);else openSelect(control);return;}}if(!target.closest("[data-dowe-select-popover]"))closeSelects();const commandClose=target.closest("[data-dowe-command-close]");if(commandClose){{event.preventDefault();closeCommand(commandClose.closest("[data-dowe-command]"));return;}}if(target.closest(".command-item")){{const command=target.closest("[data-dowe-command]");setTimeout(()=>closeCommand(command),0);}}const toastClose=target.closest("[data-dowe-toast-close]");if(toastClose){{event.preventDefault();closeToast(toastClose.closest("[data-dowe-toast]"));return;}}const navMenuTrigger=target.closest("[data-dowe-navmenu-trigger]");if(navMenuTrigger){{event.preventDefault();openNavMenu(navMenuTrigger);return;}}if(target.closest("[data-dowe-navmenu-popover] a"))closeNavMenus();if(!target.closest("[data-dowe-navmenu]"))closeNavMenus();const sideNavTrigger=target.closest(".sidenav-trigger");if(sideNavTrigger&&sideNavTrigger.closest("[data-dowe-sidenav-submenu]")){{event.preventDefault();toggleNavTreeSubmenu("sidenav",sideNavTrigger);}}const sidebarTrigger=target.closest(".sidebar-trigger");if(sidebarTrigger&&sidebarTrigger.closest("[data-dowe-sidebar-submenu]")){{event.preventDefault();toggleNavTreeSubmenu("sidebar",sidebarTrigger);}}}});
document.addEventListener("click",event=>{{const target=event.target;if(!target||!target.closest)return;const close=target.closest("[data-dowe-drawer-close]");if(close){{event.preventDefault();closeDrawer(close.closest("[data-dowe-drawer]"));return;}}const overlay=target.closest("[data-dowe-drawer-overlay]");const drawer=overlay?.closest("[data-dowe-drawer]");if(drawer&&drawer.dataset.doweDrawerDisableOverlayClose!=="true"){{event.preventDefault();closeDrawer(drawer);}}const modalClose=target.closest("[data-dowe-modal-close]");if(modalClose){{event.preventDefault();closeModal(modalClose.closest("[data-dowe-modal]"));return;}}const modalOverlay=target.closest("[data-dowe-modal-overlay]");const modal=modalOverlay?.closest("[data-dowe-modal]");if(modal&&modal.dataset.doweModalDisableOverlayClose!=="true"){{event.preventDefault();closeModal(modal);}}}});
document.addEventListener("keydown",event=>{{const command=event.target?.closest&&event.target.closest("[data-dowe-command]");if(command&&event.target.matches("[data-dowe-command-input]"))filterCommand(command);if(event.key==="Escape"){{closeSelects();closeDrawers();closeModals();closeDropdowns();closeTooltips();closeNavMenus();return;}}for(const palette of document.querySelectorAll("[data-dowe-command]")){{if(palette.dataset.doweCommandDisableGlobal==="true")continue;const mod=navigator.platform.toUpperCase().includes("MAC")?event.metaKey:event.ctrlKey;if(mod&&event.key.toLowerCase()===(palette.dataset.doweCommandShortcut||"k").toLowerCase()){{event.preventDefault();openCommand(palette);return;}}}}const tabKeys=["Enter"," ","ArrowRight","ArrowDown","ArrowLeft","ArrowUp","Home","End"];if(!tabKeys.includes(event.key))return;const target=event.target;if(!target||!target.closest)return;const tab=target.closest("[data-dowe-tab]");if(tab){{event.preventDefault();if(event.key==="Enter"||event.key===" ")setActiveTab(tab.closest("[data-dowe-tabs]"),tab.dataset.doweTab);else if(event.key==="ArrowRight"||event.key==="ArrowDown")moveActiveTab(tab,1);else if(event.key==="ArrowLeft"||event.key==="ArrowUp")moveActiveTab(tab,-1);else edgeActiveTab(tab,event.key==="End");return;}}if(event.key!=="Enter"&&event.key!==" ")return;const navMenuTrigger=target.closest("[data-dowe-navmenu-trigger]");if(navMenuTrigger){{event.preventDefault();openNavMenu(navMenuTrigger);return;}}const sideNavTrigger=target.closest(".sidenav-trigger");if(sideNavTrigger&&sideNavTrigger.closest("[data-dowe-sidenav-submenu]")){{event.preventDefault();toggleNavTreeSubmenu("sidenav",sideNavTrigger);}}const sidebarTrigger=target.closest(".sidebar-trigger");if(sidebarTrigger&&sidebarTrigger.closest("[data-dowe-sidebar-submenu]")){{event.preventDefault();toggleNavTreeSubmenu("sidebar",sidebarTrigger);}}}});
document.addEventListener("mouseenter",event=>{{const tooltip=event.target.closest&&event.target.closest("[data-dowe-tooltip]");if(tooltip)tooltipPosition(tooltip);}},true);
document.addEventListener("mouseleave",event=>{{const tooltip=event.target.closest&&event.target.closest("[data-dowe-tooltip]");if(tooltip)closeTooltips();}},true);
document.addEventListener("input",event=>{{const command=event.target.closest&&event.target.closest("[data-dowe-command]");if(command&&event.target.matches("[data-dowe-command-input]"))filterCommand(command);}});
window.addEventListener("resize",()=>{{const control=document.querySelector("[data-dowe-select].is-open");if(control)positionSelect(control);const dropdown=document.querySelector("[data-dowe-dropdown].is-open");if(dropdown)positionDropdown(dropdown);for(const carousel of document.querySelectorAll("[data-dowe-carousel]"))renderCarousel(carousel);positionOpenNavMenu();}});
window.addEventListener("scroll",()=>{{const control=document.querySelector("[data-dowe-select].is-open");if(control)positionSelect(control);const dropdown=document.querySelector("[data-dowe-dropdown].is-open");if(dropdown)positionDropdown(dropdown);positionOpenNavMenu();}},true);
document.addEventListener("click",event=>{{const actionTarget=event.target.closest&&event.target.closest("[data-dowe-click]");if(!actionTarget)return;event.preventDefault();runAction(actionTarget.dataset.doweClick,scopeFor(actionTarget));}});
document.addEventListener("click",event=>{{const button=event.target.closest&&event.target.closest("[data-dowe-code-copy]");if(!button)return;event.preventDefault();const block=button.closest("[data-dowe-code]");const source=block?.querySelector("code")?.textContent||"";const write=navigator.clipboard?.writeText?navigator.clipboard.writeText(source):Promise.reject();write.catch(()=>{{const area=document.createElement("textarea");area.value=source;document.body.appendChild(area);area.select();document.execCommand("copy");area.remove();}}).finally(()=>{{button.textContent=block?.dataset.doweCopiedLabel||"Copied";setTimeout(()=>{{button.textContent=block?.dataset.doweCopyLabel||"Copy";}},1500);}});}});
document.addEventListener("click",event=>{{const historyButton=event.target.closest&&event.target.closest("[data-dowe-history='back']");if(historyButton){{event.preventDefault();goBack();return;}}const anchor=event.target.closest&&event.target.closest("a[href]");if(!anchor||anchor.hasAttribute("download"))return;const raw=anchor.dataset.doweHref||anchor.getAttribute("href");const url=new URL(raw,location.href);if(url.protocol==="https:"&&url.origin!==location.origin)return;if(url.origin!==location.origin&&url.protocol!=="file:")return;const destination=splitDestination(raw);if(!routes[destination.path])return;event.preventDefault();navigate(destination.href,{{replace:anchor.dataset.doweNav==="replace"}});}});
if(currentFragment)scrollToFragment(currentFragment);
if(currentRoute)loadRouteModules(currentRoute).then(modules=>hydrate(currentRoute,modules));
window.doweNavigate=(path,replace=false)=>navigate(path,{{replace}});
window.doweBack=goBack;
"##
    ))
}

fn css_for_tree(tree: &ViewNode) -> String {
    let mut classes = BTreeSet::new();
    collect_classes(tree, &mut classes);
    let mut variants = Vec::new();
    collect_variant_rules(tree, &mut variants);
    let mut tabs_variants = Vec::new();
    collect_tabs_variant_rules(tree, &mut tabs_variants);
    let mut custom_rules = Vec::new();
    collect_custom_rules(tree, &mut custom_rules);
    let mut css = String::new();

    for class_name in &classes {
        append_class_css(&mut css, class_name);
    }

    for (base, family, variant) in variants {
        append_single_variant_css(&mut css, base, family, variant);
    }

    for (family, variant) in tabs_variants {
        append_tabs_variant_css(&mut css, family, variant);
    }

    for rule in custom_rules {
        css.push_str(&rule);
    }

    css
}

fn collect_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Box { props, children } => {
            classes.extend(box_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Section { props, children } => {
            classes.extend(section_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Flex { props, children } => {
            classes.extend(layout_classes("flex", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Grid { props, children } => {
            classes.extend(grid_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Card { props, children } => {
            classes.extend(variant_classes("card", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Button { props, children } => {
            classes.extend(variant_classes("button", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Avatar { props, .. } => {
            classes.extend(avatar_classes(props));
            classes.insert("avatar-image".to_string());
            classes.insert("avatar-icon".to_string());
            classes.insert("avatar-name".to_string());
            classes.insert("avatar-status".to_string());
            classes.insert("avatar-indicator".to_string());
        }
        ViewNode::Badge { props, children } => {
            classes.extend(badge_classes(props));
            classes.insert("badge-content".to_string());
            classes.insert("badge-text".to_string());
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Chip { props, .. } => {
            classes.extend(chip_classes(props));
            classes.insert("chip-label".to_string());
            classes.insert("chip-icon".to_string());
            classes.insert("chip-close".to_string());
        }
        ViewNode::Skeleton { props } => {
            classes.extend(skeleton_classes(props));
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            classes.extend(modal_panel_classes(props));
            classes.extend(modal_classes(props));
            classes.insert("modal-overlay".to_string());
            classes.insert("modal-header".to_string());
            classes.insert("modal-body".to_string());
            classes.insert("modal-footer".to_string());
            classes.insert("modal-close".to_string());
            for child in header.iter().chain(body).chain(footer) {
                collect_classes(child, classes);
            }
        }
        ViewNode::AlertDialog { props } => {
            let modal = alert_dialog_modal_props(props);
            classes.extend(modal_panel_classes(&modal));
            classes.extend(modal_classes(&modal));
            classes.insert("modal-overlay".to_string());
            classes.insert("modal-header".to_string());
            classes.insert("modal-body".to_string());
            classes.insert("modal-footer".to_string());
            classes.insert("alert-dialog-title".to_string());
            classes.insert("alert-dialog-description".to_string());
            classes.insert("alert-dialog-actions".to_string());
            classes.extend(vec![
                "button".to_string(),
                "button-md".to_string(),
                "is-outlined".to_string(),
                "is-muted".to_string(),
            ]);
        }
        ViewNode::Tooltip { props, children } => {
            classes.extend(tooltip_classes(props));
            classes.extend(tooltip_popover_classes(props));
            classes.insert("tooltip-arrow".to_string());
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Toast { props } => {
            classes.extend(toast_classes(props));
            classes.insert("toast-content".to_string());
            classes.insert("toast-title".to_string());
            classes.insert("toast-description".to_string());
            classes.insert("toast-close".to_string());
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            classes.extend(dropdown_classes(props));
            classes.extend(dropdown_popover_classes(props));
            classes.insert("dropdown-trigger".to_string());
            classes.insert("dropdown-options".to_string());
            classes.insert("dropdown-divider".to_string());
            collect_overlay_entry_classes("dropdown", entries, classes);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_classes(child, classes);
            }
        }
        ViewNode::Command { props, entries } => {
            classes.extend(command_panel_classes(props));
            classes.extend(command_classes(props));
            classes.insert("modal-overlay".to_string());
            classes.insert("command-header".to_string());
            classes.insert("command-input".to_string());
            classes.insert("command-kbd".to_string());
            classes.insert("command-results".to_string());
            classes.insert("command-empty".to_string());
            classes.insert("command-group".to_string());
            classes.insert("command-group-label".to_string());
            classes.insert("command-group-icon".to_string());
            classes.insert("command-group-items".to_string());
            classes.insert("command-shortcuts".to_string());
            collect_command_entry_classes(entries, classes);
        }
        ViewNode::Audio { props } => {
            classes.extend(variant_classes("media", &props.style));
            classes.extend([
                "media-button".to_string(),
                "media-content".to_string(),
                "media-waveform".to_string(),
                "media-bars".to_string(),
                "media-bar".to_string(),
                "media-footer".to_string(),
                "media-time".to_string(),
                "media-subtitle".to_string(),
                "media-avatar".to_string(),
            ]);
        }
        ViewNode::Image { props } => {
            classes.extend(variant_classes("image", &props.style));
            classes.extend([
                props.aspect.as_str().to_string(),
                format!("fit-{}", props.object_fit.as_str()),
                "image-element".to_string(),
                "image-controls".to_string(),
                "image-actions".to_string(),
                "image-action".to_string(),
            ]);
        }
        ViewNode::Accordion { props, items } => {
            classes.extend(variant_classes("accordion", &props.style));
            classes.extend([
                "accordion-item".to_string(),
                "accordion-header".to_string(),
                "accordion-start".to_string(),
                "accordion-label".to_string(),
                "accordion-end".to_string(),
                "accordion-arrow".to_string(),
                "accordion-content".to_string(),
            ]);
            for item in items {
                for child in &item.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            classes.extend(variant_classes("carousel", &props.style));
            classes.extend([
                "carousel-header".to_string(),
                "carousel-title".to_string(),
                "carousel-viewport".to_string(),
                "carousel-container".to_string(),
                "carousel-slide".to_string(),
                "carousel-controls".to_string(),
                "carousel-control".to_string(),
                "carousel-indicators".to_string(),
                "carousel-indicator".to_string(),
                "carousel-counter".to_string(),
                "carousel-nav".to_string(),
            ]);
            for slide in slides {
                for child in &slide.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Checkbox { props } => {
            classes.extend(["checkbox".to_string(), "checkbox-input".to_string()]);
            classes.insert(format!(
                "is-{}",
                props.style.color.unwrap_or(ColorFamily::Primary).as_str()
            ));
        }
        ViewNode::Color { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "color-field".to_string(),
                "color-input".to_string(),
                "color-field-display".to_string(),
                "color-field-swatch".to_string(),
                "color-field-value".to_string(),
                "color-picker-values".to_string(),
                "color-picker-value-code".to_string(),
            ]);
        }
        ViewNode::Date { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "date-field".to_string(),
                "date-input".to_string(),
            ]);
        }
        ViewNode::DateRange { props } => {
            classes.extend(variant_classes("control", &props.style));
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "date-range-field".to_string(),
                "date-range-inputs".to_string(),
                "date-range-separator".to_string(),
                "date-input".to_string(),
            ]);
        }
        ViewNode::RadioGroup { props, .. } => {
            classes.extend([
                "field".to_string(),
                "field-label".to_string(),
                "field-help".to_string(),
                "radio-group".to_string(),
                "radio-item".to_string(),
                "radio".to_string(),
                "label".to_string(),
                format!("is-{}", props.style.color.unwrap_or(ColorFamily::Primary).as_str()),
                format!("is-{}", props.size.as_str()),
            ]);
        }
        ViewNode::Toggle { props } => {
            classes.extend([
                "toggle".to_string(),
                "toggle-input".to_string(),
                "toggle-label-left".to_string(),
                "toggle-label-right".to_string(),
                "label-md".to_string(),
                format!("is-{}", props.style.color.unwrap_or(ColorFamily::Primary).as_str()),
            ]);
        }
        ViewNode::Input { props } => {
            classes.extend(variant_classes("control", props));
            classes.insert("input".to_string());
        }
        ViewNode::Select { props, .. } => {
            classes.extend(variant_classes("control", props));
            classes.insert("select".to_string());
            classes.insert("select-control".to_string());
            classes.insert("select-popover".to_string());
            classes.insert("select-option".to_string());
        }
        ViewNode::Code { props } => {
            classes.extend(variant_classes("code-block", &props.style));
        }
        ViewNode::Video { props } => {
            classes.extend(video_classes(props));
        }
        ViewNode::Candlestick { props } => {
            classes.extend(candlestick_classes(props));
            classes.insert("candlestick-canvas".to_string());
            classes.insert("candlestick-empty".to_string());
        }
        ViewNode::Table { props } => {
            classes.extend(table_wrapper_classes(props));
            classes.insert("table-container".to_string());
            classes.extend(table_classes(props));
            classes.insert("table-header".to_string());
            classes.insert("table-head".to_string());
            classes.insert("table-head-content".to_string());
            classes.insert("table-head-label".to_string());
            classes.insert("table-body".to_string());
            classes.insert("table-empty-row".to_string());
            classes.insert("table-empty-cell".to_string());
            classes.insert("empty-state".to_string());
            classes.insert("empty-content".to_string());
            classes.insert("empty-title".to_string());
            classes.insert("empty-description".to_string());
        }
        ViewNode::Divider { props } => {
            classes.extend(divider_classes(props));
        }
        ViewNode::Alert { props } => {
            classes.extend(variant_classes("alert", &props.style));
            classes.insert("alert-close".to_string());
        }
        ViewNode::Svg { props, .. } => {
            classes.extend(svg_classes(&props.style));
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("appbar", props, start, center, end, classes);
        }
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("footer", props, start, center, end, classes);
        }
        ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("bottombar", props, start, center, end, classes);
        }
        ViewNode::SideNav { props, items } => {
            classes.extend(side_nav_classes("sidenav", props));
            collect_side_nav_icon_classes(items, classes);
        }
        ViewNode::Sidebar { props, items } => {
            classes.extend(side_nav_classes("sidebar", props));
            collect_side_nav_icon_classes(items, classes);
        }
        ViewNode::NavMenu { props, items } => {
            classes.extend(nav_menu_classes(props));
            classes.insert("navmenu-item".to_string());
            classes.insert("navmenu-label".to_string());
            classes.insert("navmenu-icon".to_string());
            classes.insert("navmenu-arrow".to_string());
            classes.insert("navmenu-popover".to_string());
            classes.insert("navmenu-popover-content".to_string());
            classes.insert("navmenu-submenu-item".to_string());
            classes.insert("navmenu-submenu-icon".to_string());
            classes.insert("navmenu-submenu-content".to_string());
            classes.insert("navmenu-submenu-label".to_string());
            classes.insert("navmenu-submenu-description".to_string());
            collect_nav_menu_classes(items, classes);
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            classes.extend(scaffold_classes(props));
            classes.insert("scaffold-body".to_string());
            classes.insert("scaffold-main".to_string());
            classes.insert("scaffold-start".to_string());
            classes.insert("scaffold-end".to_string());
            classes.insert("scaffold-content".to_string());
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_classes(child, classes);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            classes.extend(tabs_classes(props));
            classes.extend(tabs_list_classes(props));
            classes.insert("tab".to_string());
            classes.insert("tabs-label".to_string());
            classes.insert("tabs-wrapper".to_string());
            classes.insert("tabs-content".to_string());
            for tab in tabs {
                for child in &tab.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Drawer { props, children } => {
            classes.extend(drawer_panel_classes(props));
            classes.extend(drawer_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Title { props, .. } => {
            classes.extend(text_classes("title", props));
        }
        ViewNode::Text { props, .. } => {
            classes.extend(text_classes("text", props));
        }
        ViewNode::Children => {}
    }
}

fn collect_side_nav_icon_classes(items: &[SideNavItem], classes: &mut BTreeSet<String>) {
    for item in items {
        match item {
            SideNavItem::Header(props) | SideNavItem::Item(props) => {
                collect_side_nav_item_icon_classes(props, classes);
            }
            SideNavItem::Submenu { props, items, .. } => {
                collect_side_nav_item_icon_classes(props, classes);
                for props in items {
                    collect_side_nav_item_icon_classes(props, classes);
                }
            }
            SideNavItem::Divider => {}
        }
    }
}

fn collect_side_nav_item_icon_classes(props: &SideNavItemProps, classes: &mut BTreeSet<String>) {
    if let Some(icon) = props.icon.as_ref() {
        classes.extend(svg_classes(&icon.props.style));
    }
}

fn collect_nav_menu_classes(items: &[NavMenuItem], classes: &mut BTreeSet<String>) {
    for item in items {
        match item {
            NavMenuItem::Item(props) => collect_nav_menu_item_icon_classes(props, classes),
            NavMenuItem::Submenu { props, items } => {
                collect_nav_menu_item_icon_classes(props, classes);
                for props in items {
                    collect_nav_menu_item_icon_classes(props, classes);
                }
            }
            NavMenuItem::Megamenu { props, content } => {
                collect_nav_menu_item_icon_classes(props, classes);
                for child in content {
                    collect_classes(child, classes);
                }
            }
        }
    }
}

fn collect_nav_menu_item_icon_classes(
    props: &NavMenuItemProps,
    classes: &mut BTreeSet<String>,
) {
    if let Some(icon) = props.icon.as_ref() {
        classes.extend(svg_classes(&icon.props.style));
    }
}

fn collect_bar_classes(
    base: &str,
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    classes: &mut BTreeSet<String>,
) {
    classes.extend(bar_classes(base, props));
    classes.extend(bar_content_classes(base, props));
    if !start.is_empty() {
        classes.insert(format!("{base}-start"));
    }
    if !center.is_empty() {
        classes.insert(format!("{base}-center"));
    }
    if !end.is_empty() {
        classes.insert(format!("{base}-end"));
    }
    for child in start.iter().chain(center).chain(end) {
        collect_classes(child, classes);
    }
}

fn collect_overlay_entry_classes(
    base: &str,
    entries: &[OverlayEntry],
    classes: &mut BTreeSet<String>,
) {
    for entry in entries {
        if let OverlayEntry::Item(_) = entry {
            classes.insert(format!("{base}-item"));
            classes.insert(format!("{base}-item-icon"));
            classes.insert(format!("{base}-item-content"));
            classes.insert(format!("{base}-item-label"));
            classes.insert(format!("{base}-item-description"));
        }
    }
}

fn collect_command_entry_classes(entries: &[CommandEntry], classes: &mut BTreeSet<String>) {
    for entry in entries {
        match entry {
            CommandEntry::Item(_) => {
                classes.insert("command-item".to_string());
                classes.insert("command-item-icon".to_string());
                classes.insert("command-item-content".to_string());
                classes.insert("command-item-label".to_string());
                classes.insert("command-item-description".to_string());
            }
            CommandEntry::Group { items, .. } => {
                for _ in items {
                    classes.insert("command-item".to_string());
                    classes.insert("command-item-icon".to_string());
                    classes.insert("command-item-content".to_string());
                    classes.insert("command-item-label".to_string());
                    classes.insert("command-item-description".to_string());
                }
            }
        }
    }
}

fn collect_variant_rules<'a>(
    node: &'a ViewNode,
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    match node {
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. } => {
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Card { props, children } => {
            push_variant_rule(variants, "card", props);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Drawer { props, children } => {
            push_variant_rule(variants, "drawer", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Avatar { props, .. } => push_variant_rule(variants, "avatar", &props.style),
        ViewNode::Badge { props, children } => {
            push_variant_rule(variants, "badge", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Chip { props, .. } => push_variant_rule(variants, "chip", &props.style),
        ViewNode::Skeleton { .. } => {}
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            push_variant_rule(variants, "modal", &props.style);
            for child in header.iter().chain(body).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::AlertDialog { props } => {
            push_variant_rule(variants, "modal", &props.style);
            let cancel = ("button", ColorFamily::Muted, ComponentVariant::Outlined);
            if !variants.contains(&cancel) {
                variants.push(cancel);
            }
            let confirm = (
                "button",
                props.style.color.unwrap_or(ColorFamily::Danger),
                ComponentVariant::Solid,
            );
            if !variants.contains(&confirm) {
                variants.push(confirm);
            }
        }
        ViewNode::Tooltip { props, children } => {
            push_variant_rule(variants, "tooltip-popover", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Toast { props } => push_variant_rule(variants, "toast", &props.style),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            push_variant_rule(variants, "dropdown-popover", &props.style);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Command { props, .. } => push_variant_rule(variants, "command", &props.style),
        ViewNode::Audio { props } => push_variant_rule(variants, "media", &props.style),
        ViewNode::Image { props } => push_variant_rule(variants, "image", &props.style),
        ViewNode::Accordion { props, items } => {
            push_variant_rule(variants, "accordion", &props.style);
            for item in items {
                for child in &item.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            push_variant_rule(variants, "carousel", &props.style);
            for slide in slides {
                for child in &slide.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Checkbox { .. } => {}
        ViewNode::Color { props } => push_variant_rule(variants, "control", &props.style),
        ViewNode::Date { props } => push_variant_rule(variants, "control", &props.style),
        ViewNode::DateRange { props } => push_variant_rule(variants, "control", &props.style),
        ViewNode::RadioGroup { .. } => {}
        ViewNode::Toggle { .. } => {}
        ViewNode::Button { props, children } => {
            push_variant_rule(variants, "button", props);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Input { props } => {
            push_variant_rule(variants, "control", props);
        }
        ViewNode::Select { props, .. } => {
            push_variant_rule(variants, "control", props);
        }
        ViewNode::Code { props } => {
            push_variant_rule(variants, "code-block", &props.style);
        }
        ViewNode::Video { props } => {
            push_variant_rule(variants, "video", &props.style);
        }
        ViewNode::Candlestick { props } => {
            push_variant_rule(variants, "candlestick", &props.style);
        }
        ViewNode::Table { props } => {
            push_variant_rule(variants, "table", &props.style);
        }
        ViewNode::Divider { .. } => {}
        ViewNode::Alert { props } => {
            push_variant_rule(variants, "alert", &props.style);
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => {
            push_variant_rule(variants, "appbar", &props.style);
            for child in start.iter().chain(center).chain(end) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        } => {
            push_variant_rule(variants, "footer", &props.style);
            for child in start.iter().chain(center).chain(end) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            push_variant_rule(variants, "bottombar", &props.style);
            for child in start.iter().chain(center).chain(end) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::SideNav { props, .. } => {
            push_variant_rule(variants, "sidenav", &props.style);
        }
        ViewNode::Sidebar { props, .. } => {
            push_variant_rule(variants, "sidebar", &props.style);
        }
        ViewNode::NavMenu { props, items } => {
            push_variant_rule(variants, "navmenu", &props.style);
            for item in items {
                collect_nav_menu_variant_rules(item, variants);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Svg { .. } => {}
        ViewNode::Title { .. } | ViewNode::Text { .. } | ViewNode::Children => {}
    }
}

fn collect_nav_menu_variant_rules(
    item: &NavMenuItem,
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_variant_rules(child, variants);
        }
    }
}

fn collect_tabs_variant_rules(node: &ViewNode, variants: &mut Vec<(ColorFamily, TabsVariant)>) {
    match node {
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Button { children, .. } => {
            for child in children {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Tabs { props, tabs } => {
            let rule = (props.color, props.variant);
            if !variants.contains(&rule) {
                variants.push(rule);
            }
            for tab in tabs {
                for child in &tab.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::AppBar {
            start,
            center,
            end,
            ..
        }
        | ViewNode::Footer {
            start,
            center,
            end,
            ..
        }
        | ViewNode::BottomBar {
            start,
            center,
            end,
            ..
        } => {
            for child in start.iter().chain(center).chain(end) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                collect_nav_menu_tabs_variant_rules(item, variants);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Children => {}
    }
}

fn collect_nav_menu_tabs_variant_rules(
    item: &NavMenuItem,
    variants: &mut Vec<(ColorFamily, TabsVariant)>,
) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_tabs_variant_rules(child, variants);
        }
    }
}

fn collect_custom_rules(node: &ViewNode, rules: &mut Vec<String>) {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_style_custom_rules(props, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Flex { props, children } => {
            collect_gap_custom_rules(props.gap.as_ref(), rules);
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Grid { props, children } => {
            collect_grid_custom_rules(props, rules);
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Card { props, children } => {
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Drawer { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Avatar { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Badge { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Chip { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Skeleton { props } => collect_style_custom_rules(&props.style, rules),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in header.iter().chain(body).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::AlertDialog { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Tooltip { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Toast { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Command { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Audio { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Image { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Accordion { props, items } => {
            collect_style_custom_rules(&props.style.style, rules);
            for item in items {
                for child in &item.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            collect_style_custom_rules(&props.style.style, rules);
            for slide in slides {
                for child in &slide.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Checkbox { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Color { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Date { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::DateRange { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::RadioGroup { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Toggle { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Button { props, children } => {
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            collect_style_custom_rules(&props.style, rules)
        }
        ViewNode::Code { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Video { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Candlestick { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Table { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Divider { props } => {
            collect_divider_custom_rules(props, rules);
            collect_style_custom_rules(&props.style, rules);
        }
        ViewNode::Alert { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in start.iter().chain(center).chain(end) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::SideNav { props, .. } => {
            collect_style_custom_rules(&props.style.style, rules);
        }
        ViewNode::Sidebar { props, .. } => {
            collect_style_custom_rules(&props.style.style, rules);
        }
        ViewNode::NavMenu { props, items } => {
            collect_style_custom_rules(&props.style.style, rules);
            for item in items {
                collect_nav_menu_custom_rules(item, rules);
            }
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            collect_style_custom_rules(&props.style, rules);
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            collect_style_custom_rules(&props.style, rules);
            for tab in tabs {
                for child in &tab.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Svg { props, .. } => collect_style_custom_rules(&props.style, rules),
        ViewNode::Title { props, .. } | ViewNode::Text { props, .. } => {
            collect_style_custom_rules(&props.style, rules)
        }
        ViewNode::Children => {}
    }
}

fn collect_nav_menu_custom_rules(item: &NavMenuItem, rules: &mut Vec<String>) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_custom_rules(child, rules);
        }
    }
}

fn collect_divider_custom_rules(props: &DividerProps, rules: &mut Vec<String>) {
    let token = props.color.as_str();
    let rule = format!(
        ".divider.is-{token}{{background-color:var(--dowe-{token});color:var(--dowe-{token});}}"
    );
    if !rules.contains(&rule) {
        rules.push(rule);
    }
}

fn collect_style_custom_rules(props: &StyleProps, rules: &mut Vec<String>) {
    if let Some(cover) = props.cover.as_ref() {
        for entry in &cover.entries {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{background-image:url(\"{}\");}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("cover-{}", cover_suffix(&entry.value))
                    )),
                    escape_css_string(&entry.value.0)
                ),
            );
        }
    }
    if let Some(overlay) = props.overlay.as_ref() {
        for entry in &overlay.entries {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}::before{{background:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("overlay-{}", overlay_suffix(&entry.value))
                    )),
                    overlay_css(&entry.value)
                ),
            );
        }
    }
}

fn collect_gap_custom_rules(value: Option<&ResponsiveValue<GapValue>>, rules: &mut Vec<String>) {
    let Some(value) = value else {
        return;
    };
    for entry in &value.entries {
        if let GapValue::Pair(row, column) = &entry.value {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{row-gap:{};column-gap:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("gap-{}", entry.value.class_suffix())
                    )),
                    gap_size_css(row),
                    gap_size_css(column)
                ),
            );
        }
    }
}

fn collect_grid_custom_rules(props: &GridProps, rules: &mut Vec<String>) {
    collect_gap_custom_rules(props.gap.as_ref(), rules);
    collect_grid_track_rules(
        "grid-cols",
        "grid-template-columns",
        props.columns.as_ref(),
        rules,
    );
    collect_grid_track_rules(
        "grid-rows",
        "grid-template-rows",
        props.rows.as_ref(),
        rules,
    );
}

fn collect_grid_track_rules(
    prefix: &str,
    property: &str,
    value: Option<&ResponsiveValue<GridTracks>>,
    rules: &mut Vec<String>,
) {
    let Some(value) = value else {
        return;
    };
    for entry in &value.entries {
        if let GridTracks::Template(template) = &entry.value {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{{property}:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("{prefix}-{}", entry.value.class_suffix())
                    )),
                    template
                ),
            );
        }
    }
}
