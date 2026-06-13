fn dev_activity_svg_parser() -> &'static str {
    r#"    private static final class DoweSvgPathEntry {
        private final String data;
        private final boolean currentColor;
        private final Integer color;

        DoweSvgPathEntry(String data, boolean currentColor, Integer color) {
            this.data = data;
            this.currentColor = currentColor;
            this.color = color;
        }
    }

    private static final class DoweSvgPathParser {
        private final String source;
        private int index = 0;
        private char command = 0;

        private DoweSvgPathParser(String source) {
            this.source = source;
        }

        private static Path parse(String source) {
            return new DoweSvgPathParser(source).readPath();
        }

        private Path readPath() {
            Path path = new Path();
            float currentX = 0f;
            float currentY = 0f;
            float startX = 0f;
            float startY = 0f;
            float lastCubicX = 0f;
            float lastCubicY = 0f;
            float lastQuadX = 0f;
            float lastQuadY = 0f;
            boolean hasLastCubic = false;
            boolean hasLastQuad = false;

            while (true) {
                skipSeparators();
                if (index >= source.length()) {
                    return path;
                }
                char next = source.charAt(index);
                if (isCommand(next)) {
                    command = next;
                    index++;
                } else if (command == 0) {
                    return path;
                }
                boolean relative = Character.isLowerCase(command);
                switch (Character.toUpperCase(command)) {
                    case 'M': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        currentX = coordinate(x, currentX, relative);
                        currentY = coordinate(y, currentY, relative);
                        path.moveTo(currentX, currentY);
                        startX = currentX;
                        startY = currentY;
                        command = relative ? 'l' : 'L';
                        while (hasNumber()) {
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                            currentX = coordinate(x, currentX, relative);
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'L': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            currentX = coordinate(x, currentX, relative);
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'H': {
                        Float x = readNumber();
                        if (x == null) {
                            return path;
                        }
                        while (true) {
                            currentX = coordinate(x, currentX, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            if (x == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'V': {
                        Float y = readNumber();
                        if (y == null) {
                            return path;
                        }
                        while (true) {
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            y = readNumber();
                            if (y == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'C': {
                        Float x1 = readNumber();
                        Float y1 = readNumber();
                        Float x2 = readNumber();
                        Float y2 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x1 == null || y1 == null || x2 == null || y2 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float control1X = coordinate(x1, currentX, relative);
                            float control1Y = coordinate(y1, currentY, relative);
                            float control2X = coordinate(x2, currentX, relative);
                            float control2Y = coordinate(y2, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.cubicTo(control1X, control1Y, control2X, control2Y, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastCubicX = control2X;
                            lastCubicY = control2Y;
                            hasLastCubic = true;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x1 = readNumber();
                            y1 = readNumber();
                            x2 = readNumber();
                            y2 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x1 == null || y1 == null || x2 == null || y2 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'S': {
                        Float x2 = readNumber();
                        Float y2 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x2 == null || y2 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float control1X = hasLastCubic ? 2f * currentX - lastCubicX : currentX;
                            float control1Y = hasLastCubic ? 2f * currentY - lastCubicY : currentY;
                            float control2X = coordinate(x2, currentX, relative);
                            float control2Y = coordinate(y2, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.cubicTo(control1X, control1Y, control2X, control2Y, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastCubicX = control2X;
                            lastCubicY = control2Y;
                            hasLastCubic = true;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x2 = readNumber();
                            y2 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x2 == null || y2 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'Q': {
                        Float x1 = readNumber();
                        Float y1 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x1 == null || y1 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float controlX = coordinate(x1, currentX, relative);
                            float controlY = coordinate(y1, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.quadTo(controlX, controlY, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastQuadX = controlX;
                            lastQuadY = controlY;
                            hasLastQuad = true;
                            hasLastCubic = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x1 = readNumber();
                            y1 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x1 == null || y1 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'T': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float controlX = hasLastQuad ? 2f * currentX - lastQuadX : currentX;
                            float controlY = hasLastQuad ? 2f * currentY - lastQuadY : currentY;
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.quadTo(controlX, controlY, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastQuadX = controlX;
                            lastQuadY = controlY;
                            hasLastQuad = true;
                            hasLastCubic = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'A': {
                        Float rx = readNumber();
                        Float ry = readNumber();
                        Float angle = readNumber();
                        Float largeArc = readNumber();
                        Float sweep = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (rx == null || ry == null || angle == null || largeArc == null || sweep == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            addArc(path, currentX, currentY, rx, ry, angle, largeArc != 0f, sweep != 0f, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            hasLastCubic = false;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            rx = readNumber();
                            ry = readNumber();
                            angle = readNumber();
                            largeArc = readNumber();
                            sweep = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (rx == null || ry == null || angle == null || largeArc == null || sweep == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'Z':
                        path.close();
                        currentX = startX;
                        currentY = startY;
                        hasLastCubic = false;
                        hasLastQuad = false;
                        command = 0;
                        break;
                    default:
                        return path;
                }
            }
        }

        private boolean hasNumber() {
            skipSeparators();
            if (index >= source.length()) {
                return false;
            }
            char value = source.charAt(index);
            return Character.isDigit(value) || value == '+' || value == '-' || value == '.';
        }

        private Float readNumber() {
            skipSeparators();
            int start = index;
            if (index < source.length() && (source.charAt(index) == '+' || source.charAt(index) == '-')) {
                index++;
            }
            boolean digits = false;
            while (index < source.length() && Character.isDigit(source.charAt(index))) {
                index++;
                digits = true;
            }
            if (index < source.length() && source.charAt(index) == '.') {
                index++;
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                    digits = true;
                }
            }
            if (!digits) {
                index = start;
                return null;
            }
            if (index < source.length() && (source.charAt(index) == 'e' || source.charAt(index) == 'E')) {
                int exponent = index;
                index++;
                if (index < source.length() && (source.charAt(index) == '+' || source.charAt(index) == '-')) {
                    index++;
                }
                int exponentDigits = index;
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                }
                if (index == exponentDigits) {
                    index = exponent;
                }
            }
            try {
                return Float.parseFloat(source.substring(start, index));
            } catch (NumberFormatException error) {
                return null;
            }
        }

        private void skipSeparators() {
            while (index < source.length()) {
                char value = source.charAt(index);
                if (Character.isWhitespace(value) || value == ',') {
                    index++;
                } else {
                    return;
                }
            }
        }

        private static boolean isCommand(char value) {
            return "MmZzLlHhVvCcSsQqTtAa".indexOf(value) >= 0;
        }

        private static float coordinate(float value, float current, boolean relative) {
            return relative ? current + value : value;
        }

        private static void addArc(Path path, float currentX, float currentY, float rawRx, float rawRy, float angle, boolean largeArc, boolean sweep, float endX, float endY) {
            double rx = Math.abs(rawRx);
            double ry = Math.abs(rawRy);
            if (rx == 0 || ry == 0 || (currentX == endX && currentY == endY)) {
                path.lineTo(endX, endY);
                return;
            }
            double phi = Math.toRadians(angle);
            double cosPhi = Math.cos(phi);
            double sinPhi = Math.sin(phi);
            double dx = (currentX - endX) / 2.0;
            double dy = (currentY - endY) / 2.0;
            double x1p = cosPhi * dx + sinPhi * dy;
            double y1p = -sinPhi * dx + cosPhi * dy;
            double lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
            if (lambda > 1) {
                double factor = Math.sqrt(lambda);
                rx *= factor;
                ry *= factor;
            }
            double rx2 = rx * rx;
            double ry2 = ry * ry;
            double denominator = rx2 * y1p * y1p + ry2 * x1p * x1p;
            if (denominator == 0) {
                path.lineTo(endX, endY);
                return;
            }
            double sign = largeArc == sweep ? -1 : 1;
            double factor = sign * Math.sqrt(Math.max(0, (rx2 * ry2 - rx2 * y1p * y1p - ry2 * x1p * x1p) / denominator));
            double cxp = factor * rx * y1p / ry;
            double cyp = factor * -ry * x1p / rx;
            double cx = cosPhi * cxp - sinPhi * cyp + (currentX + endX) / 2.0;
            double cy = sinPhi * cxp + cosPhi * cyp + (currentY + endY) / 2.0;
            double theta = vectorAngle(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry);
            double delta = vectorAngle((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry);
            if (!sweep && delta > 0) {
                delta -= 2 * Math.PI;
            } else if (sweep && delta < 0) {
                delta += 2 * Math.PI;
            }
            int segments = Math.max(1, (int) Math.ceil(Math.abs(delta) / (Math.PI / 2)));
            double step = delta / segments;
            for (int segment = 0; segment < segments; segment++) {
                double next = theta + step;
                addArcSegment(path, cx, cy, rx, ry, phi, theta, next);
                theta = next;
            }
        }

        private static void addArcSegment(Path path, double cx, double cy, double rx, double ry, double phi, double start, double end) {
            double alpha = 4.0 / 3.0 * Math.tan((end - start) / 4.0);
            double cosStart = Math.cos(start);
            double sinStart = Math.sin(start);
            double cosEnd = Math.cos(end);
            double sinEnd = Math.sin(end);
            float[] control1 = arcPoint(cx, cy, rx, ry, phi, cosStart - alpha * sinStart, sinStart + alpha * cosStart);
            float[] control2 = arcPoint(cx, cy, rx, ry, phi, cosEnd + alpha * sinEnd, sinEnd - alpha * cosEnd);
            float[] point = arcPoint(cx, cy, rx, ry, phi, cosEnd, sinEnd);
            path.cubicTo(control1[0], control1[1], control2[0], control2[1], point[0], point[1]);
        }

        private static float[] arcPoint(double cx, double cy, double rx, double ry, double phi, double x, double y) {
            return new float[] {
                (float) (cx + rx * Math.cos(phi) * x - ry * Math.sin(phi) * y),
                (float) (cy + rx * Math.sin(phi) * x + ry * Math.cos(phi) * y)
            };
        }

        private static double vectorAngle(double ux, double uy, double vx, double vy) {
            double length = Math.sqrt((ux * ux + uy * uy) * (vx * vx + vy * vy));
            if (length == 0) {
                return 0;
            }
            double value = Math.max(-1, Math.min(1, (ux * vx + uy * vy) / length));
            double sign = ux * vy - uy * vx < 0 ? -1 : 1;
            return sign * Math.acos(value);
        }
    }

"#
}
