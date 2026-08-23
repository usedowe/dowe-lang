        HashMap<String, Object> args = new HashMap<>();
        if (action.stdlibArgs != null) {
            for (Object[] arg : action.stdlibArgs) {
                args.put(String.valueOf(arg[0]), doweStdlibValue((Object[]) arg[1], item));
            }
        }
        String name = action.stdlibNamespace + "." + action.stdlibFunction;
        if ("str.trim".equals(name)) return doweStdlibText(args.get("value")).trim();
        if ("str.lower".equals(name)) return doweStdlibText(args.get("value")).toLowerCase();
        if ("str.upper".equals(name)) return doweStdlibText(args.get("value")).toUpperCase();
        if ("str.length".equals(name)) return doweStdlibText(args.get("value")).codePointCount(0, doweStdlibText(args.get("value")).length());
        if ("str.contains".equals(name)) return doweStdlibText(args.get("value")).contains(doweStdlibText(args.get("needle")));
        if ("str.startsWith".equals(name)) return doweStdlibText(args.get("value")).startsWith(doweStdlibText(args.get("prefix")));
        if ("str.endsWith".equals(name)) return doweStdlibText(args.get("value")).endsWith(doweStdlibText(args.get("suffix")));
        if ("str.replace".equals(name)) return doweStdlibText(args.get("value")).replace(doweStdlibText(args.get("from")), doweStdlibText(args.get("to")));
        if ("str.split".equals(name)) return doweStdlibSplit(doweStdlibText(args.get("value")), doweStdlibText(args.get("delimiter")), args.get("limit"));
        if ("str.join".equals(name)) return String.join(doweStdlibText(args.get("delimiter")), doweStdlibList(args.get("values")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("math.add".equals(name)) return doweFinite(args.get("left"), args.get("right"), '+');
        if ("math.sub".equals(name)) return doweFinite(args.get("left"), args.get("right"), '-');
        if ("math.mul".equals(name)) return doweFinite(args.get("left"), args.get("right"), '*');
        if ("math.div".equals(name)) return doweFinite(args.get("left"), args.get("right"), '/');
        if ("math.round".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.round(doweStdlibNumber(args.get("value")));
        if ("math.floor".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.floor(doweStdlibNumber(args.get("value")));
        if ("math.ceil".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.ceil(doweStdlibNumber(args.get("value")));
        if ("math.abs".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.abs(doweStdlibNumber(args.get("value")));
        if ("math.sum".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).reduce(0.0, Double::sum);
        if ("math.average".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).collect(java.util.stream.Collectors.averagingDouble(value -> value));
        if ("math.min".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).min(Double::compareTo).orElse(null);
        if ("math.max".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).max(Double::compareTo).orElse(null);
        if ("parse.int".equals(name)) {
            try {
                return Long.parseLong(doweStdlibText(args.get("value")).trim());
            } catch (NumberFormatException error) {
                return args.get("fallback");
            }
        }
        if ("parse.float".equals(name)) return doweStdlibNumber(args.get("value")) == null ? args.get("fallback") : doweStdlibNumber(args.get("value"));
        if ("parse.bool".equals(name)) return doweStdlibBool(args.get("value")) == null ? args.get("fallback") : doweStdlibBool(args.get("value"));
        if ("parse.string".equals(name)) return doweStdlibText(args.get("value"));
        if ("parse.svg".equals(name)) return doweParseSvg(doweStdlibText(args.get("value")), args.get("fallback"), args.containsKey("colors") ? doweStdlibText(args.get("colors")) : "tokens", args.containsKey("format") ? doweStdlibText(args.get("format")) : "source");
        if ("parse.json".equals(name) || "json.parse".equals(name)) {
            try { return doweFromJson(new org.json.JSONTokener(doweStdlibText(args.get("value"))).nextValue()); } catch (Exception error) { return args.get("fallback"); }
        }
        if ("url.encode".equals(name)) {
            try { return java.net.URLEncoder.encode(doweStdlibText(args.get("value")), java.nio.charset.StandardCharsets.UTF_8.name()).replace("+", "%20"); } catch (Exception error) { return doweStdlibText(args.get("value")); }
        }
        if ("url.decode".equals(name)) {
            try { return java.net.URLDecoder.decode(doweStdlibText(args.get("value")), java.nio.charset.StandardCharsets.UTF_8.name()); } catch (Exception error) { return args.get("fallback"); }
        }
        if ("url.parse".equals(name)) return doweStdlibUrlParse(doweStdlibText(args.get("value")));
        if ("url.queryGet".equals(name)) return android.net.Uri.parse(doweStdlibText(args.get("value"))).getQueryParameter(doweStdlibText(args.get("name")));
        if ("url.querySet".equals(name)) return doweStdlibUrlQuerySet(doweStdlibText(args.get("value")), doweStdlibText(args.get("name")), args.get("param"));
        if ("csv.parse".equals(name)) return doweStdlibCsvParse(doweStdlibText(args.get("value")), doweStdlibText(args.get("delimiter")).isEmpty() ? "," : doweStdlibText(args.get("delimiter")), Boolean.TRUE.equals(args.get("header")), doweStdlibNumber(args.get("maxRows")) == null ? 1000 : doweStdlibNumber(args.get("maxRows")).intValue(), doweStdlibNumber(args.get("maxColumns")) == null ? 100 : doweStdlibNumber(args.get("maxColumns")).intValue());
        if ("csv.stringify".equals(name)) return doweStdlibCsvStringify(doweStdlibList(args.get("rows")), doweStdlibText(args.get("delimiter")).isEmpty() ? "," : doweStdlibText(args.get("delimiter")));
        if ("sort.asc".equals(name)) return doweSorted(args.get("values"), null, false, "last");
        if ("sort.desc".equals(name)) return doweSorted(args.get("values"), null, true, "last");
        if ("sort.by".equals(name)) return doweSorted(args.get("values"), doweStdlibText(args.get("field")), "desc".equals(doweStdlibText(args.get("direction"))), doweStdlibText(args.get("nulls")));
        if ("list.take".equals(name)) return new ArrayList<>(doweStdlibList(args.get("values")).subList(0, Math.min(doweStdlibList(args.get("values")).size(), Math.max(0, doweStdlibNumber(args.get("count")).intValue()))));
        if ("list.skip".equals(name)) return new ArrayList<>(doweStdlibList(args.get("values")).subList(Math.min(doweStdlibList(args.get("values")).size(), Math.max(0, doweStdlibNumber(args.get("count")).intValue())), doweStdlibList(args.get("values")).size()));
        if ("list.first".equals(name)) return doweStdlibList(args.get("values")).isEmpty() ? null : doweStdlibList(args.get("values")).get(0);
        if ("list.last".equals(name)) return doweStdlibList(args.get("values")).isEmpty() ? null : doweStdlibList(args.get("values")).get(doweStdlibList(args.get("values")).size() - 1);
        if ("list.count".equals(name)) return doweStdlibList(args.get("values")).size();
        if ("list.filterEquals".equals(name)) return doweStdlibList(args.get("values")).stream().filter(value -> Objects.equals(doweStdlibRead(value, doweStdlibText(args.get("field"))), args.get("value"))).collect(java.util.stream.Collectors.toList());
        if ("list.filterContains".equals(name)) return doweStdlibList(args.get("values")).stream().filter(value -> doweStdlibText(doweStdlibRead(value, doweStdlibText(args.get("field")))).toLowerCase().contains(doweStdlibText(args.get("value")).toLowerCase())).collect(java.util.stream.Collectors.toList());
        if ("list.mapField".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibRead(value, doweStdlibText(args.get("field")))).collect(java.util.stream.Collectors.toList());
        if ("list.sumBy".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibNumber(doweStdlibRead(value, doweStdlibText(args.get("field"))))).filter(Objects::nonNull).reduce(0.0, Double::sum);
        if ("list.averageBy".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibNumber(doweStdlibRead(value, doweStdlibText(args.get("field"))))).filter(Objects::nonNull).collect(java.util.stream.Collectors.averagingDouble(value -> value));
        if ("json.get".equals(name)) {
            Object value = doweStdlibRead(args.get("value"), doweStdlibText(args.get("path")));
            return value == null ? args.get("fallback") : value;
        }
        if ("json.set".equals(name)) return doweStdlibSet(args.get("value"), doweStdlibText(args.get("path")), args.get("next"));
        if ("json.pick".equals(name)) return doweStdlibPick(args.get("value"), doweStdlibList(args.get("fields")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("json.omit".equals(name)) return doweStdlibOmit(args.get("value"), doweStdlibList(args.get("fields")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("json.stringify".equals(name)) return doweJsonString(args.get("value"), Boolean.TRUE.equals(args.get("pretty")));
        if ("json.merge".equals(name)) return doweStdlibMerge(args.get("left"), args.get("right"));
        if ("date.now".equals(name)) return java.time.Instant.now().toString();
        if ("date.formatIso".equals(name)) { try { return java.time.Instant.parse(doweStdlibText(args.get("value"))).toString(); } catch (Exception error) { return doweStdlibText(args.get("value")); } }
        if ("date.addDays".equals(name)) { try { return java.time.Instant.parse(doweStdlibText(args.get("value"))).plus(java.time.Duration.ofDays(doweStdlibNumber(args.get("days")) == null ? 0 : doweStdlibNumber(args.get("days")).longValue())).toString(); } catch (Exception error) { return null; } }
        if ("date.diffDays".equals(name)) { try { return java.time.Duration.between(java.time.Instant.parse(doweStdlibText(args.get("start"))), java.time.Instant.parse(doweStdlibText(args.get("end")))).toDays(); } catch (Exception error) { return 0L; } }
        return null;
    }

    private Object doweStdlibValue(Object[] value, Map<String, Object> item) {
        String kind = String.valueOf(value[0]);
        Object raw = value[1];
        if ("null".equals(kind)) return null;
        if ("bool".equals(kind)) return raw;
        if ("number".equals(kind)) return doweStdlibNumber(raw);
        if ("string".equals(kind)) return raw == null ? "" : String.valueOf(raw);
        if ("reference".equals(kind)) return doweRead(String.valueOf(raw), item);
        if ("array".equals(kind)) {
            ArrayList<Object> result = new ArrayList<>();
            for (Object entry : (Object[]) raw) result.add(doweStdlibValue((Object[]) entry, item));
            return result;
        }
        return raw;
    }

    private String doweStdlibText(Object value) {
        return value == null ? "" : String.valueOf(value);
    }

    private Double doweStdlibNumber(Object value) {
        if (value instanceof Number) return ((Number) value).doubleValue();
        try {
            return Double.parseDouble(doweStdlibText(value).trim());
        } catch (NumberFormatException error) {
            return null;
        }
    }

    private Boolean doweStdlibBool(Object value) {
        if (value instanceof Boolean) return (Boolean) value;
        String text = doweStdlibText(value).trim().toLowerCase(java.util.Locale.ROOT);
        if ("true".equals(text) || "1".equals(text) || "yes".equals(text) || "y".equals(text)) return true;
        if ("false".equals(text) || "0".equals(text) || "no".equals(text) || "n".equals(text)) return false;
        return null;
    }

    private Map<String, Object> doweStdlibUrlParse(String value) {
        HashMap<String, Object> result = new HashMap<>();
        try {
            android.net.Uri uri = android.net.Uri.parse(value);
            HashMap<String, String> query = new HashMap<>();
            for (String key : uri.getQueryParameterNames()) query.put(key, uri.getQueryParameter(key));
            result.put("ok", true);
            result.put("scheme", uri.getScheme());
            result.put("host", uri.getHost());
            result.put("path", uri.getPath() == null ? "" : uri.getPath());
            result.put("query", query);
            result.put("fragment", uri.getFragment());
            result.put("origin", uri.getScheme() == null || uri.getHost() == null ? null : uri.getScheme() + "://" + uri.getHost());
            result.put("isRelative", uri.getScheme() == null);
            result.put("error", null);
        } catch (Exception error) {
            result.put("ok", false);
            result.put("scheme", null);
            result.put("host", null);
            result.put("path", null);
            result.put("query", new HashMap<String, String>());
            result.put("fragment", null);
            result.put("origin", null);
            result.put("isRelative", false);
            result.put("error", "invalid_url");
        }
        return result;
    }

    private String doweStdlibUrlQuerySet(String value, String name, Object param) {
        try {
            android.net.Uri uri = android.net.Uri.parse(value);
            android.net.Uri.Builder builder = uri.buildUpon().clearQuery();
            for (String key : uri.getQueryParameterNames()) {
                if (name.equals(key)) continue;
                for (String item : uri.getQueryParameters(key)) builder.appendQueryParameter(key, item);
            }
            if (param != null) builder.appendQueryParameter(name, doweStdlibText(param));
            return builder.build().toString();
        } catch (Exception error) {
            return value;
        }
    }

    private Map<String, Object> doweStdlibCsvParse(String value, String delimiter, boolean header, int maxRows, int maxColumns) {
        char separator = delimiter.isEmpty() ? ',' : delimiter.charAt(0);
        ArrayList<List<String>> parsed = new ArrayList<>();
        ArrayList<String> row = new ArrayList<>();
        StringBuilder cell = new StringBuilder();
        boolean quoted = false;
        boolean truncated = false;
        for (int index = 0; index < value.length(); index++) {
            char character = value.charAt(index);
            if (character == '"' && quoted && index + 1 < value.length() && value.charAt(index + 1) == '"') {
                cell.append('"');
                index++;
            } else if (character == '"') {
                quoted = !quoted;
            } else if (!quoted && character == separator) {
                row.add(cell.toString());
                cell.setLength(0);
            } else if (!quoted && (character == '\n' || character == '\r')) {
                row.add(cell.toString());
                cell.setLength(0);
                if (parsed.size() < Math.max(0, maxRows)) parsed.add(new ArrayList<>(row.subList(0, Math.min(row.size(), Math.max(0, maxColumns))))); else truncated = true;
                row.clear();
                if (character == '\r' && index + 1 < value.length() && value.charAt(index + 1) == '\n') index++;
            } else {
                cell.append(character);
            }
        }
        if (cell.length() > 0 || !row.isEmpty()) {
            row.add(cell.toString());
            if (parsed.size() < Math.max(0, maxRows)) parsed.add(new ArrayList<>(row.subList(0, Math.min(row.size(), Math.max(0, maxColumns))))); else truncated = true;
        }
        ArrayList<String> columns = new ArrayList<>();
        int width = parsed.stream().mapToInt(List::size).max().orElse(0);
        if (header && !parsed.isEmpty()) columns.addAll(parsed.get(0)); else for (int index = 0; index < width; index++) columns.add("column" + (index + 1));
        ArrayList<Object> rows = new ArrayList<>();
        for (int index = header && !parsed.isEmpty() ? 1 : 0; index < parsed.size(); index++) {
            List<String> values = parsed.get(index);
            if (header) {
                HashMap<String, Object> object = new HashMap<>();
                for (int column = 0; column < columns.size(); column++) object.put(columns.get(column), column < values.size() ? values.get(column) : "");
                rows.add(object);
            } else {
                rows.add(values);
            }
        }
        HashMap<String, Object> result = new HashMap<>();
        result.put("rows", rows);
        result.put("columns", columns);
        result.put("errors", new ArrayList<>());
        result.put("truncated", truncated);
        result.put("rowCount", rows.size());
        return result;
    }

    private String doweStdlibCsvStringify(List<Object> rows, String delimiter) {
        String separator = delimiter.isEmpty() ? "," : delimiter.substring(0, 1);
        ArrayList<String> columns = new ArrayList<>();
        if (!rows.isEmpty() && rows.get(0) instanceof Map) for (Object key : ((Map<?, ?>) rows.get(0)).keySet()) columns.add(String.valueOf(key));
        java.util.Collections.sort(columns);
        ArrayList<String> output = new ArrayList<>();
        for (Object row : rows) {
            ArrayList<String> values = new ArrayList<>();
            if (row instanceof Map) for (String key : columns) values.add(doweStdlibCsvEscape(((Map<?, ?>) row).get(key), separator));
            else if (row instanceof List) for (Object value : (List<?>) row) values.add(doweStdlibCsvEscape(value, separator));
            else values.add(doweStdlibCsvEscape(row, separator));
            output.add(String.join(separator, values));
        }
        return String.join("\n", output);
    }

    private String doweStdlibCsvEscape(Object value, String delimiter) {
        String text = doweStdlibText(value);
        return text.contains(delimiter) || text.contains("\"") || text.contains("\n") || text.contains("\r") ? "\"" + text.replace("\"", "\"\"") + "\"" : text;
    }

    private Object doweStdlibSet(Object value, String path, Object next) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        String[] parts = path.split("\\.");
        if (parts.length == 0 || parts[0].isEmpty()) return next;
        Map<String, Object> current = result;
        for (int index = 0; index < parts.length - 1; index++) {
            HashMap<String, Object> child = new HashMap<>();
            if (current.get(parts[index]) instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) current.get(parts[index])).entrySet()) child.put(String.valueOf(entry.getKey()), entry.getValue());
            current.put(parts[index], child);
            current = child;
        }
        current.put(parts[parts.length - 1], next);
        return result;
    }

    private Map<String, Object> doweStdlibPick(Object value, List<String> fields) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (String field : fields) if (((Map<?, ?>) value).containsKey(field)) result.put(field, ((Map<?, ?>) value).get(field));
        return result;
    }

    private Map<String, Object> doweStdlibOmit(Object value, List<String> fields) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) if (!fields.contains(String.valueOf(entry.getKey()))) result.put(String.valueOf(entry.getKey()), entry.getValue());
        return result;
    }

    private Map<String, Object> doweStdlibMerge(Object left, Object right) {
        HashMap<String, Object> result = new HashMap<>();
        if (left instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) left).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        if (right instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) right).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        return result;
    }

    private List<Object> doweStdlibList(Object value) {
        return value instanceof List ? (List<Object>) value : new ArrayList<>();
    }

    private String[] doweStdlibSplit(String value, String delimiter, Object limitValue) {
        String[] parts = value.split(java.util.regex.Pattern.quote(delimiter), -1);
        Double limit = doweStdlibNumber(limitValue);
        if (limit == null) return parts;
        return java.util.Arrays.copyOf(parts, Math.min(parts.length, Math.max(0, limit.intValue())));
    }

    private Double doweFinite(Object leftValue, Object rightValue, char operation) {
        Double left = doweStdlibNumber(leftValue);
        Double right = doweStdlibNumber(rightValue);
        if (left == null || right == null) return null;
        if (operation == '+') return left + right;
        if (operation == '-') return left - right;
        if (operation == '*') return left * right;
        if (operation == '/') return right == 0.0 ? null : left / right;
        return null;
    }

    private Object doweStdlibRead(Object value, String path) {
        Object current = value;
        for (String part : path.split("\\.")) {
            if (!(current instanceof Map)) return null;
            current = ((Map<?, ?>) current).get(part);
        }
        return current;
    }

    private ArrayList<Object> doweSorted(Object value, String field, boolean desc, String nulls) {
        ArrayList<Object> result = new ArrayList<>(doweStdlibList(value));
        result.sort((left, right) -> {
            Object leftValue = field == null ? left : doweStdlibRead(left, field);
            Object rightValue = field == null ? right : doweStdlibRead(right, field);
            boolean leftNull = leftValue == null;
            boolean rightNull = rightValue == null;
            if (leftNull || rightNull) {
                if (leftNull && rightNull) return 0;
                return leftNull == !"first".equals(nulls) ? 1 : -1;
            }
            int order = doweStdlibText(leftValue).compareTo(doweStdlibText(rightValue));
            return desc ? -order : order;
        });
        return result;
    }

    private void doweRunAction(String id, Map<String, Object> item) {
