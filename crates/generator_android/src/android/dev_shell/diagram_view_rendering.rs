r#"        @Override
        protected void onDraw(android.graphics.Canvas canvas) {
            canvas.drawColor(surfaceColor);
            refreshNodes();
            if (showGrid && scale > 0.15f) {
                float step = doweDp(28) * scale;
                if (step > 6f) {
                    for (float x = offsetX % step; x < getWidth(); x += step) canvas.drawLine(x, 0f, x, getHeight(), gridPaint);
                    for (float y = offsetY % step; y < getHeight(); y += step) canvas.drawLine(0f, y, getWidth(), y, gridPaint);
                }
            }
            List<Map<String, Object>> edges = diagramEdges();
            for (Map<String, Object> edge : edges) {
                Map<String, Object> source = edge.get("source") == null ? null : findNode(String.valueOf(edge.get("source")));
                Map<String, Object> target = edge.get("target") == null ? null : findNode(String.valueOf(edge.get("target")));
                if (source == null || target == null) continue;
                PointF sourceCenter = nodeCenter(source);
                PointF targetCenter = nodeCenter(target);
                PointF from = borderPoint(source, targetCenter.x, targetCenter.y);
                PointF to = borderPoint(target, sourceCenter.x, sourceCenter.y);
                String type = edge.get("type") == null ? "default" : String.valueOf(edge.get("type"));
                android.graphics.Path path = new android.graphics.Path();
                PointF labelPoint = new PointF();
                float fromX = toScreenX(from.x), fromY = toScreenY(from.y);
                float toX = toScreenX(to.x), toY = toScreenY(to.y);
                if ("straight".equals(type)) {
                    path.moveTo(fromX, fromY);
                    path.lineTo(toX, toY);
                    labelPoint.set((fromX + toX) / 2f, (fromY + toY) / 2f);
                } else if ("step".equals(type)) {
                    float midX = (fromX + toX) / 2f;
                    path.moveTo(fromX, fromY);
                    path.lineTo(midX, fromY);
                    path.lineTo(midX, toY);
                    path.lineTo(toX, toY);
                    labelPoint.set(midX, (fromY + toY) / 2f);
                } else {
                    float dx = Math.max(doweDp(40), Math.abs(toX - fromX) / 2f);
                    path.moveTo(fromX, fromY);
                    path.cubicTo(fromX + dx, fromY, toX - dx, toY, toX, toY);
                    labelPoint.set((fromX + toX) / 2f, (fromY + toY) / 2f);
                }
                boolean isSelected = selectedKey != null && selectedKey.equals("edge:" + String.valueOf(edge.get("id")));
                edgePaint.setColor(contentColor);
                edgePaint.setAlpha(isSelected ? 255 : 115);
                edgePaint.setStrokeWidth(doweDp(isSelected ? 3 : 2));
                canvas.drawPath(path, edgePaint);
                Object label = edge.get("label");
                if (label != null && !String.valueOf(label).isEmpty())
                    canvas.drawText(String.valueOf(label), labelPoint.x, labelPoint.y - doweDp(6), edgeTextPaint);
            }
            if (connectId != null && findNode(connectId) != null) {
                Map<String, Object> source = findNode(connectId);
                PointF from = borderPoint(source, connectX, connectY);
                float fromX = toScreenX(from.x), fromY = toScreenY(from.y);
                float toX = toScreenX(connectX), toY = toScreenY(connectY);
                float dx = Math.max(doweDp(40), Math.abs(toX - fromX) / 2f);
                android.graphics.Path preview = new android.graphics.Path();
                preview.moveTo(fromX, fromY);
                preview.cubicTo(fromX + dx, fromY, toX - dx, toY, toX, toY);
                canvas.drawPath(preview, previewPaint);
            }
            for (int index = 0; index < nodes.size(); index++) {
                Map<String, Object> node = nodes.get(index);
                float nx = toScreenX(number(node, "x", 0f));
                float ny = toScreenY(number(node, "y", 0f));
                float width = nodeWidth(node) * scale;
                float height = nodeHeight(node) * scale;
                boolean isSelected = selectedKey != null && selectedKey.equals("node:" + String.valueOf(node.get("id")));
                Map<String, Object> connectTarget = connectId == null ? null : hitNodeAtGraph(connectX, connectY);
                boolean isTarget = connectId != null && !connectId.equals(String.valueOf(node.get("id"))) && connectTarget != null && String.valueOf(connectTarget.get("id")).equals(String.valueOf(node.get("id")));
                nodePaint.setPathEffect(isTarget ? new android.graphics.DashPathEffect(new float[]{doweDp(5), doweDp(3)}, 0f) : null);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(isSelected || isTarget ? 41 : 20);
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                nodePaint.setStyle(android.graphics.Paint.Style.STROKE);
                nodePaint.setStrokeWidth(doweDp(isSelected || isTarget ? 2 : 1));
                nodePaint.setAlpha(isSelected || isTarget ? 255 : 90);
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                nodePaint.setPathEffect(null);
                Object label = node.get("label") != null ? node.get("label") : node.get("id");
                textPaint.setTextSize(spToPx(13) * scale);
                canvas.drawText(String.valueOf(label), nx + width / 2f, ny + height / 2f - (textPaint.ascent() + textPaint.descent()) / 2f, textPaint);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(255);
                canvas.drawCircle(nx + width, ny + height / 2f, doweDp(5), nodePaint);
            }
            if (nodes.isEmpty() && !emptyLabel.isEmpty()) {
                textPaint.setTextSize(spToPx(13));
                canvas.drawText(emptyLabel, getWidth() / 2f, getHeight() / 2f - (textPaint.ascent() + textPaint.descent()) / 2f, textPaint);
            }
            if (minimap && !nodes.isEmpty()) drawMinimap(canvas);
            if (controls) drawControls(canvas);
        }

        private void drawMinimap(android.graphics.Canvas canvas) {
            android.graphics.RectF rect = minimapRect();
            canvas.drawRoundRect(rect, doweDp(8), doweDp(8), minimapViewStrokePaint);
            float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE, maxX = -Float.MAX_VALUE, maxY = -Float.MAX_VALUE;
            for (Map<String, Object> node : nodes) {
                minX = Math.min(minX, number(node, "x", 0f));
                minY = Math.min(minY, number(node, "y", 0f));
                maxX = Math.max(maxX, number(node, "x", 0f) + nodeWidth(node));
                maxY = Math.max(maxY, number(node, "y", 0f) + nodeHeight(node));
            }
            float graphWidth = Math.max(1f, maxX - minX);
            float graphHeight = Math.max(1f, maxY - minY);
            float padding = doweDp(8);
            float fit = Math.min((rect.width() - padding * 2f) / graphWidth, (rect.height() - padding * 2f) / graphHeight);
            for (Map<String, Object> node : nodes) {
                android.graphics.RectF nodeRect = new android.graphics.RectF(
                    rect.left + (number(node, "x", 0f) - minX) * fit + padding,
                    rect.top + (number(node, "y", 0f) - minY) * fit + padding,
                    rect.left + (number(node, "x", 0f) - minX) * fit + padding + Math.max(doweDp(3), nodeWidth(node) * fit),
                    rect.top + (number(node, "y", 0f) - minY) * fit + padding + Math.max(doweDp(2), nodeHeight(node) * fit));
                canvas.drawRoundRect(nodeRect, doweDp(2), doweDp(2), minimapNodePaint);
            }
            android.graphics.RectF viewRect = new android.graphics.RectF(
                rect.left + (-offsetX / scale - minX) * fit + padding,
                rect.top + (-offsetY / scale - minY) * fit + padding,
                rect.left + (-offsetX / scale - minX) * fit + padding + (getWidth() / scale) * fit,
                rect.top + (-offsetY / scale - minY) * fit + padding + (getHeight() / scale) * fit);
            canvas.drawRect(viewRect, minimapViewFillPaint);
            canvas.drawRect(viewRect, minimapViewStrokePaint);
        }

        private void drawControls(android.graphics.Canvas canvas) {
            String[] labels = {"+", "−", "⤢"};
            android.graphics.Paint buttonPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            android.graphics.Paint buttonStroke = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            android.graphics.Paint buttonText = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            buttonPaint.setColor(surfaceColor);
            buttonPaint.setAlpha(242);
            buttonStroke.setStyle(android.graphics.Paint.Style.STROKE);
            buttonStroke.setColor(contentColor);
            buttonStroke.setAlpha(38);
            buttonStroke.setStrokeWidth(doweDp(1));
            buttonText.setColor(contentColor);
            buttonText.setTextSize(spToPx(14));
            buttonText.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.BOLD));
            buttonText.setTextAlign(android.graphics.Paint.Align.CENTER);
            for (int index = 0; index < 3; index++) {
                android.graphics.RectF rect = controlRect(index);
                canvas.drawRoundRect(rect, doweDp(8), doweDp(8), buttonPaint);
                canvas.drawRoundRect(rect, doweDp(8), doweDp(8), buttonStroke);
                canvas.drawText(labels[index], rect.centerX(), rect.centerY() - (buttonText.ascent() + buttonText.descent()) / 2f, buttonText);
            }
        }

        private void moveViewportToMinimap(float x, float y) {
            android.graphics.RectF rect = minimapRect();
            float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE, maxX = -Float.MAX_VALUE, maxY = -Float.MAX_VALUE;
            for (Map<String, Object> node : nodes) {
                minX = Math.min(minX, number(node, "x", 0f));
                minY = Math.min(minY, number(node, "y", 0f));
                maxX = Math.max(maxX, number(node, "x", 0f) + nodeWidth(node));
                maxY = Math.max(maxY, number(node, "y", 0f) + nodeHeight(node));
            }
            float graphWidth = Math.max(1f, maxX - minX);
            float graphHeight = Math.max(1f, maxY - minY);
            float padding = doweDp(8);
            float fit = Math.min((rect.width() - padding * 2f) / graphWidth, (rect.height() - padding * 2f) / graphHeight);
            float graphX = (x - rect.left - padding) / fit + minX;
            float graphY = (y - rect.top - padding) / fit + minY;
            offsetX = getWidth() / 2f - graphX * scale;
            offsetY = getHeight() / 2f - graphY * scale;
            invalidate();
        }

"#
