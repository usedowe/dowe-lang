        HashMap<String, Object> result = new HashMap<>();
        for (int index = 0; index + 1 < values.length; index += 2) {
            result.put((String) values[index], values[index + 1]);
        }
        return result;
    }

    private ArrayList<Object> doweArray(Object... values) {
        ArrayList<Object> result = new ArrayList<>();
        for (Object value : values) {
            result.add(value);
        }
        return result;
    }

    private void dowePutSignalMetadata(String id, String name, String scope, String storage) {
        doweSignalMetadata.put(id, new String[] { name, scope, storage });
        if ("global".equals(scope)) {
            doweGlobalStorage.put(name, storage);
        }
    }

    private void dowePutForm(String signal, DoweFormFieldMetadata[] fields) {
        doweForms.put(signal, fields == null ? new DoweFormFieldMetadata[0] : fields);
    }

    private void dowePutInitial(String path, Object value) {
        doweInitial.put(path, doweCopy(value));
        String[] metadata = doweSignalMetadata.get(path);
        if (metadata != null && "global".equals(metadata[1])) {
            if (!doweGlobalState.containsKey(metadata[0])) {
                Object stored = doweStoredSignal(metadata[0], metadata[2]);
                doweGlobalState.put(metadata[0], stored == null ? doweCopy(value) : stored);
            }
            doweState.put(path, doweCopy(doweGlobalState.get(metadata[0])));
        } else {
            doweState.put(path, doweCopy(value));
        }
    }

    private Object doweStoredSignal(String name, String storage) {
        if (!"local".equals(storage)) {
            return null;
        }
        String raw = getSharedPreferences("dowe_view_state", MODE_PRIVATE).getString("dowe:signal:" + name, null);
        if (raw == null) {
            return null;
        }
        try {
            JSONObject wrapper = new JSONObject(raw);
            return doweFromJson(wrapper.get("value"));
        } catch (Exception error) {
            return null;
        }
    }

    private void dowePersistRoot(String root) {
        String[] metadata = doweSignalMetadata.get(root);
        if (metadata == null || !"global".equals(metadata[1])) {
            return;
        }
        Object value = doweState.get(root);
        doweGlobalState.put(metadata[0], doweCopy(value));
        if ("local".equals(metadata[2])) {
            try {
                JSONObject wrapper = new JSONObject();
                wrapper.put("value", doweJson(value));
                getSharedPreferences("dowe_view_state", MODE_PRIVATE).edit().putString("dowe:signal:" + metadata[0], wrapper.toString()).apply();
            } catch (Exception error) {
            }
        }
    }

    private Object doweCopy(Object value) {
        if (value instanceof Map) {
            HashMap<String, Object> result = new HashMap<>();
            for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) {
                result.put(String.valueOf(entry.getKey()), doweCopy(entry.getValue()));
            }
            return result;
        }
        if (value instanceof List) {
            ArrayList<Object> result = new ArrayList<>();
            for (Object item : (List<?>) value) {
                result.add(doweCopy(item));
            }
            return result;
        }
        return value;
    }

    private Object doweRead(String path, Map<String, Object> item) {
        Object derived = doweFormValue(path, item);
        if (derived != null) {
            return derived;
        }
        if ("item".equals(path)) {
            return item;
        }
        if (path.startsWith("item.") && item != null) {
            return doweReadMap(path.substring(5), item);
        }
        return doweReadMap(path, doweState);
    }

    private String doweFormError(String form, DoweFormFieldMetadata field, Map<String, Object> item) {
        Object value = doweReadMap(form + "." + field.path, doweState);
        String[][] rules = new String[field.rules.length][3];
        for (int index = 0; index < field.rules.length; index++) {
            String[] rule = field.rules[index];
            rules[index][0] = rule.length > 0 ? rule[0] : "";
            rules[index][1] = rule.length > 1 && rule[1] != null ? doweTextValue(rule[1], item) : null;
            rules[index][2] = rule.length > 2 ? rule[2] : "";
        }
        return doweValidationError(value == null ? "" : String.valueOf(value), rules, field.booleanValue);
    }

    private Object doweFormValue(String path, Map<String, Object> item) {
        String[] parts = path == null ? new String[0] : path.split("\\.");
        if (parts.length < 2) {
            return null;
        }
        DoweFormFieldMetadata[] fields = doweForms.get(parts[0]);
        if (fields == null) {
            return null;
        }
        String form = parts[0];
        if ("isValid".equals(parts[1]) && parts.length == 2) {
            for (DoweFormFieldMetadata field : fields) if (doweFormError(form, field, item) != null) return false;
            return true;
        }
        if ("isInvalid".equals(parts[1]) && parts.length == 2) {
            for (DoweFormFieldMetadata field : fields) if (doweFormError(form, field, item) != null) return true;
            return false;
        }
        if ("errors".equals(parts[1])) {
            HashMap<String, Object> errors = new HashMap<>();
            for (DoweFormFieldMetadata field : fields) {
                String error = doweFormError(form, field, item);
                if (error != null && !error.isEmpty()) errors.put(field.path, error);
            }
            return parts.length == 2 ? errors : errors.get(String.join(".", java.util.Arrays.copyOfRange(parts, 2, parts.length)));
        }
        if ("touched".equals(parts[1])) {
            HashMap<String, Object> touched = new HashMap<>();
            for (DoweFormFieldMetadata field : fields) touched.put(field.path, doweTouchedForms.contains(form + "." + field.path));
            return parts.length == 2 ? touched : doweTouchedForms.contains(form + "." + String.join(".", java.util.Arrays.copyOfRange(parts, 2, parts.length)));
        }
        return null;
    }

    private boolean doweValidateForm(String form) {
        DoweFormFieldMetadata[] fields = doweForms.get(form);
        if (fields == null) return true;
        for (DoweFormFieldMetadata field : fields) {
            doweTouchedForms.add(form + "." + field.path);
        }
        for (DoweValidationBinding binding : doweControlValidations.values()) binding.touch();
        return Boolean.TRUE.equals(doweFormValue(form + ".isValid", null));
    }

    private Object doweReadMap(String path, Map<String, Object> values) {
        String[] parts = path.split("\\.");
        Object current = values.get(parts[0]);
        for (int index = 1; index < parts.length; index++) {
            if (!(current instanceof Map)) {
                return null;
            }
            current = ((Map<?, ?>) current).get(parts[index]);
        }
        return current;
    }

    private String doweTextValue(String path, Map<String, Object> item) {
        Object value = doweRead(path, item);
        return value == null ? "" : String.valueOf(value);
    }

    private boolean doweBool(String path) {
        return doweBool(path, null);
    }

    private boolean doweBool(String path, Map<String, Object> item) {
        Object value = doweRead(path, item);
        return value instanceof Boolean && (Boolean) value;
    }

    private int doweButtonFamily(String scheme) {
