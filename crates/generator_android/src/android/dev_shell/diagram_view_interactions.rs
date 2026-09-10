r#"        private boolean hasCurrentNode(Object id) {
            Object current = doweRead(nodesPath, null);
            if (!(current instanceof List)) return false;
            for (Object entry : (List<?>) current) {
                if (entry instanceof Map && ((Map<?, ?>) entry).get("id") != null && String.valueOf(id).equals(String.valueOf(((Map<?, ?>) entry).get("id")))) return true;
            }
            return false;
        }

        private boolean persistConnection(Object source, Object target) {
            if (source == null || target == null || String.valueOf(source).equals(String.valueOf(target)) || !hasCurrentNode(source) || !hasCurrentNode(target)) return false;
            Object current = doweRead(edgesPath, null);
            ArrayList<Object> edges = current instanceof List ? new ArrayList<>((List<?>) current) : new ArrayList<>();
            java.util.Set<String> usedIds = new java.util.HashSet<>();
            for (Object entry : edges) {
                if (!(entry instanceof Map)) continue;
                Map<?, ?> row = (Map<?, ?>) entry;
                if (String.valueOf(source).equals(String.valueOf(row.get("source"))) && String.valueOf(target).equals(String.valueOf(row.get("target")))) return false;
                if (row.get("id") != null) usedIds.add(String.valueOf(row.get("id")));
            }
            int sequence = 1;
            while (usedIds.contains("edge-" + sequence)) sequence++;
            Map<String, Object> edge = new HashMap<>();
            edge.put("id", "edge-" + sequence);
            edge.put("source", source);
            edge.put("target", target);
            edge.put("type", "default");
            edge.put("label", "");
            edges.add(edge);
            doweWrite(edgesPath, edges);
            invalidate();
            return true;
        }

        private String hitEdge(float graphX, float graphY) {
            for (Map<String, Object> edge : diagramEdges()) {
                Map<String, Object> source = edge.get("source") == null ? null : findNode(String.valueOf(edge.get("source")));
                Map<String, Object> target = edge.get("target") == null ? null : findNode(String.valueOf(edge.get("target")));
                if (source == null || target == null) continue;
                PointF sourceCenter = nodeCenter(source);
                PointF targetCenter = nodeCenter(target);
                PointF from = borderPoint(source, targetCenter.x, targetCenter.y);
                PointF to = borderPoint(target, sourceCenter.x, sourceCenter.y);
                if (distanceToSegment(graphX, graphY, from.x, from.y, to.x, to.y) <= doweDp(8) / Math.max(scale, 0.01f))
                    return String.valueOf(edge.get("id"));
            }
            return null;
        }

        private float distanceToSegment(float px, float py, float ax, float ay, float bx, float by) {
            float abx = bx - ax;
            float aby = by - ay;
            float lengthSquared = abx * abx + aby * aby;
            if (lengthSquared == 0f) return (float) Math.hypot(px - ax, py - ay);
            float t = Math.max(0f, Math.min(1f, ((px - ax) * abx + (py - ay) * aby) / lengthSquared));
            return (float) Math.hypot(px - (ax + t * abx), py - (ay + t * aby));
        }

        private boolean isTapSlop() {
            return Math.abs(downX - lastX) < doweDp(6) && Math.abs(downY - lastY) < doweDp(6);
        }

        private void resetMode() {
            mode = MODE_IDLE;
            dragId = null;
            connectId = null;
            dragMoved = false;
            controlTarget = -1;
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            scaleDetector.onTouchEvent(event);
            if (scaleDetector.isInProgress()) {
                resetMode();
                refreshNodes();
                invalidate();
                return true;
            }
            int action = event.getActionMasked();
            if (action == MotionEvent.ACTION_DOWN) {
                downX = lastX = event.getX();
                downY = lastY = event.getY();
                resetMode();
                refreshNodes();
                if (minimap && !nodes.isEmpty() && minimapRect().contains(event.getX(), event.getY())) {
                    mode = MODE_MINIMAP;
                    moveViewportToMinimap(event.getX(), event.getY());
                } else if (hitControl(event.getX(), event.getY()) >= 0) {
                    mode = MODE_CONTROL;
                    controlTarget = hitControl(event.getX(), event.getY());
                } else {
                    int hit = hitNode(event.getX(), event.getY());
                    if (hit >= 0) {
                        Map<String, Object> node = nodes.get(hit);
                        float right = toScreenX(number(node, "x", 0f)) + nodeWidth(node) * scale;
                        if (event.getX() >= right - doweDp(18)) {
                            connectId = String.valueOf(node.get("id"));
                            PointF center = nodeCenter(node);
                            connectX = center.x + nodeWidth(node) / 2f;
                            connectY = center.y;
                            mode = MODE_CONNECT;
                        } else {
                            dragId = String.valueOf(node.get("id"));
                            dragX = number(node, "x", 0f);
                            dragY = number(node, "y", 0f);
                            mode = MODE_DRAG;
                        }
                    } else if (panOnDrag) {
                        mode = MODE_PAN;
                    }
                }
                invalidate();
                return true;
            }
            if (action == MotionEvent.ACTION_MOVE) {
                float deltaX = event.getX() - lastX;
                float deltaY = event.getY() - lastY;
                lastX = event.getX();
                lastY = event.getY();
                if (mode == MODE_MINIMAP) {
                    moveViewportToMinimap(event.getX(), event.getY());
                } else if (mode == MODE_CONNECT && connectId != null) {
                    connectX = toGraphX(event.getX());
                    connectY = toGraphY(event.getY());
                    invalidate();
                } else if (mode == MODE_DRAG && dragId != null) {
                    dragX += deltaX / scale;
                    dragY += deltaY / scale;
                    if (Math.hypot(event.getX() - downX, event.getY() - downY) > doweDp(6)) dragMoved = true;
                    if (dragMoved) {
                        refreshNodes();
                        invalidate();
                    }
                } else if (mode == MODE_PAN) {
                    offsetX += deltaX;
                    offsetY += deltaY;
                    invalidate();
                }
                return true;
            }
            if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_CANCEL) {
                if (action == MotionEvent.ACTION_CANCEL) {
                    resetMode();
                    refreshNodes();
                    invalidate();
                    return true;
                }
                lastX = event.getX();
                lastY = event.getY();
                if (mode == MODE_CONTROL && controlTarget >= 0) {
                    if (controlTarget == 0) zoomAtCenter(1.2f);
                    else if (controlTarget == 1) zoomAtCenter(1f / 1.2f);
                    else { applyFitView(); invalidate(); }
                } else if (mode == MODE_CONNECT && connectId != null) {
                    Map<String, Object> target = hitNodeAtGraph(toGraphX(event.getX()), toGraphY(event.getY()));
                    if (target != null && connectId.equals(String.valueOf(target.get("id")))) target = null;
                    if (target != null) {
                        Object sourceId = connectId;
                        Object targetId = target.get("id");
                        if (persistConnection(sourceId, targetId) && connectAction != null && actionSink != null) {
                            Map<String, Object> item = new HashMap<>();
                            item.put("source", sourceId);
                            item.put("target", targetId);
                            actionSink.run(connectAction, item);
                        }
                    }
                } else if (mode == MODE_DRAG && dragId != null) {
                    if (dragMoved) {
                        Map<String, Object> item = writeDraggedNode();
                        if (item != null) {
                            selectedKey = "node:" + dragId;
                            if (nodeDragAction != null && actionSink != null) actionSink.run(nodeDragAction, item);
                        }
                    } else if (isTapSlop()) {
                        Map<String, Object> node = findNode(dragId);
                        if (node != null && hasCurrentNode(dragId)) {
                            selectedKey = "node:" + dragId;
                            if (nodeClickAction != null && actionSink != null) actionSink.run(nodeClickAction, new HashMap<>(node));
                        }
                    }
                } else if ((mode == MODE_IDLE || mode == MODE_PAN) && isTapSlop()) {
                    String edge = hitEdge(toGraphX(event.getX()), toGraphY(event.getY()));
                    selectedKey = edge == null ? null : "edge:" + edge;
                }
                resetMode();
                invalidate();
                return true;
            }
            return true;
        }
    }
"#
