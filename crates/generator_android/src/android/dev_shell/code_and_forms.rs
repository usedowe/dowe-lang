fn dev_activity_code_and_forms() -> String {
    let mut output = String::new();
    output.push_str(dev_activity_code_and_forms_code_and_text());
    output.push_str("__DOWE_ANDROID_DEV_FONT_SUPPORT__");
    output.push_str(dev_activity_code_and_forms_validation());
    output.push_str(dev_activity_code_and_forms_rich_text_countdown());
    output.push_str(dev_activity_code_and_forms_inputs_select());
    output.push_str(dev_activity_code_and_forms_combo_layout());
    output.push_str(dev_activity_code_and_forms_color());
    output.push_str(dev_activity_code_and_forms_date_phone());
    output.push_str("__DOWE_JAVA_REACTIVE_RUNTIME__");
    output
}
