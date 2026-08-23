        private final String kind;
        private final String method;
        private final String path;
        private final String base;
        private final Object[][] headers;
        private final String body;
        private final String update;
        private final String reset;
        private final String successAlert;
        private final String successMessage;
        private final String errorAlert;
        private final String errorMessage;
        private final String target;
        private final String source;
        private final String stdlibNamespace;
        private final String stdlibFunction;
        private final Object[][] stdlibArgs;
        private final DoweStep[] steps;

        private DoweAction(String kind, String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage, String target, String source, String stdlibNamespace, String stdlibFunction, Object[][] stdlibArgs) {
            this(kind, method, path, base, headers, body, update, reset, successAlert, successMessage, errorAlert, errorMessage, target, source, stdlibNamespace, stdlibFunction, stdlibArgs, null);
        }

        private DoweAction(String kind, String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage, String target, String source, String stdlibNamespace, String stdlibFunction, Object[][] stdlibArgs, DoweStep[] steps) {
            this.kind = kind;
            this.method = method;
            this.path = path;
            this.base = base;
            this.headers = headers == null ? new Object[0][0] : headers;
            this.body = body;
            this.update = update;
            this.reset = reset;
            this.successAlert = successAlert;
            this.successMessage = successMessage;
            this.errorAlert = errorAlert;
            this.errorMessage = errorMessage;
            this.target = target;
            this.source = source;
            this.stdlibNamespace = stdlibNamespace;
            this.stdlibFunction = stdlibFunction;
            this.stdlibArgs = stdlibArgs;
            this.steps = steps;
        }

        private static DoweAction request(String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage) {
            return new DoweAction("request", method, path, base, headers, body, update, reset, successAlert, successMessage, errorAlert, errorMessage, null, null, null, null, null);
        }

        private static DoweAction assign(String target, String source) {
            return new DoweAction("assign", null, null, null, null, null, null, null, null, null, null, null, target, source, null, null, null);
        }

        private static DoweAction assignCall(String target, String source, String namespace, String function, Object[][] args) {
            return new DoweAction("assign", null, null, null, null, null, null, null, null, null, null, null, target, source, namespace, function, args);
        }

        private static DoweAction reset(String target) {
            return new DoweAction("reset", null, null, null, null, null, null, null, null, null, null, null, target, null, null, null, null);
        }

        private static DoweAction sequence(DoweStep[] steps) {
            return new DoweAction("sequence", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, steps);
        }
    }

    private static final class DoweStep {
        private final String kind;
        private final String result;
        private final DoweAction request;
        private final DoweStep[] success;
        private final DoweStep[] error;
        private final String target;
        private final String source;
        private final Object literal;
        private final boolean hasLiteral;
        private final DoweAction call;
        private final String title;
        private final String message;
        private final Integer duration;
        private final String scheme;
        private final String variant;
        private final String position;

        private DoweStep(String kind, String result, DoweAction request, DoweStep[] success, DoweStep[] error, String target, String source, Object literal, boolean hasLiteral, DoweAction call, String title, String message, Integer duration, String scheme, String variant, String position) {
            this.kind = kind; this.result = result; this.request = request; this.success = success; this.error = error;
            this.target = target; this.source = source; this.literal = literal; this.hasLiteral = hasLiteral; this.call = call;
            this.title = title; this.message = message; this.duration = duration;
            this.scheme = scheme; this.variant = variant; this.position = position;
        }

        private static DoweStep request(String result, DoweAction action) { return new DoweStep("request", result, action, null, null, null, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep validate(String target) { return new DoweStep("validate", null, null, null, null, target, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep branch(String result, DoweStep[] success, DoweStep[] error) { return new DoweStep("branch", result, null, success, error, null, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep assign(String target, String source, Object literal, boolean hasLiteral, DoweAction call) { return new DoweStep("assign", null, null, null, null, target, source, literal, hasLiteral, call, null, null, null, null, null, null); }
        private static DoweStep reset(String target) { return new DoweStep("reset", null, null, null, null, target, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep toast(String kind, String title, String message, Integer duration, String scheme, String variant, String position) { return new DoweStep(kind, null, null, null, null, null, null, null, false, null, title, message, duration, scheme, variant, position); }
        private static DoweStep redirect(String path) { return new DoweStep("redirect", null, null, null, null, path, null, null, false, null, null, null, null, null, null, null); }
    }

    private static final class DoweFormFieldMetadata {
        private final String path;
        private final boolean booleanValue;
        private final String[][] rules;

        private DoweFormFieldMetadata(String path, boolean booleanValue, String[][] rules) {
            this.path = path;
            this.booleanValue = booleanValue;
            this.rules = rules == null ? new String[0][0] : rules;
        }
    }

    private HashMap<String, Object> doweObject(Object... values) {
