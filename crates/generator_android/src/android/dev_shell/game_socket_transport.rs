fn dev_activity_game_socket_transport() -> &'static str {
    r#"        private Socket openTransport(String target) throws Exception {
            URI uri = new URI(target);
            String scheme = uri.getScheme() == null ? "" : uri.getScheme().toLowerCase(Locale.US);
            if (!("ws".equals(scheme) || "wss".equals(scheme)) || uri.getHost() == null || uri.getUserInfo() != null) {
                throw new java.io.IOException("Invalid WebSocket URL");
            }
            int port = uri.getPort();
            if (port < 0) port = "wss".equals(scheme) ? 443 : 80;
            Socket connection;
            if ("wss".equals(scheme)) {
                SSLSocket secure = (SSLSocket) SSLSocketFactory.getDefault().createSocket();
                javax.net.ssl.SSLParameters parameters = secure.getSSLParameters();
                parameters.setEndpointIdentificationAlgorithm("HTTPS");
                secure.setSSLParameters(parameters);
                secure.connect(new InetSocketAddress(uri.getHost(), port), 10000);
                secure.startHandshake();
                connection = secure;
            } else {
                connection = new Socket();
                connection.connect(new InetSocketAddress(uri.getHost(), port), 10000);
            }
            connection.setSoTimeout(0);
            handshake(connection, uri, scheme, port);
            return connection;
        }

        private void handshake(Socket connection, URI uri, String scheme, int port) throws Exception {
            byte[] nonce = new byte[16];
            random.nextBytes(nonce);
            String key = Base64.encodeToString(nonce, Base64.NO_WRAP);
            String path = uri.getRawPath();
            if (path == null || path.length() == 0) path = "/";
            if (uri.getRawQuery() != null) path += "?" + uri.getRawQuery();
            String host = uri.getHost();
            if (host.indexOf(':') >= 0) host = "[" + host + "]";
            boolean defaultPort = ("ws".equals(scheme) && port == 80) || ("wss".equals(scheme) && port == 443);
            if (!defaultPort) host += ":" + port;
            String request = "GET " + path + " HTTP/1.1\r\n"
                + "Host: " + host + "\r\n"
                + "Upgrade: websocket\r\n"
                + "Connection: Upgrade\r\n"
                + "Sec-WebSocket-Key: " + key + "\r\n"
                + "Sec-WebSocket-Version: 13\r\n\r\n";
            java.io.OutputStream stream = connection.getOutputStream();
            stream.write(request.getBytes(java.nio.charset.StandardCharsets.US_ASCII));
            stream.flush();
            InputStream input = connection.getInputStream();
            String status = readLine(input);
            if (!(status.startsWith("HTTP/1.1 101") || status.startsWith("HTTP/1.0 101"))) {
                throw new java.io.IOException("WebSocket handshake rejected: " + status);
            }
            String accept = null;
            String line;
            while ((line = readLine(input)).length() > 0) {
                int separator = line.indexOf(':');
                if (separator > 0 && "sec-websocket-accept".equalsIgnoreCase(line.substring(0, separator).trim())) {
                    accept = line.substring(separator + 1).trim();
                }
            }
            String expected = Base64.encodeToString(
                MessageDigest.getInstance("SHA-1").digest((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").getBytes(java.nio.charset.StandardCharsets.US_ASCII)),
                Base64.NO_WRAP
            );
            if (!expected.equals(accept)) throw new java.io.IOException("Invalid WebSocket handshake");
        }

        private String readLine(InputStream input) throws java.io.IOException {
            StringBuilder line = new StringBuilder();
            int value;
            while ((value = input.read()) >= 0) {
                if (value == '\n') {
                    if (line.length() > 0 && line.charAt(line.length() - 1) == '\r') line.deleteCharAt(line.length() - 1);
                    return line.toString();
                }
                if (line.length() >= 16384) throw new java.io.IOException("WebSocket handshake line too long");
                line.append((char) value);
            }
            throw new java.io.IOException("WebSocket handshake closed");
        }

        private boolean claim(Socket connection, long expectedGeneration) throws java.io.IOException {
            synchronized (this) {
                if (stopped || generation != expectedGeneration) return false;
                transport = connection;
                output = connection.getOutputStream();
                return true;
            }
        }

        private boolean isCurrent(Socket connection, long expectedGeneration) {
            synchronized (this) {
                return !stopped && generation == expectedGeneration && transport == connection;
            }
        }

        private void readLoop(Socket connection, long expectedGeneration) throws java.io.IOException {
            InputStream input = connection.getInputStream();
            java.io.ByteArrayOutputStream message = null;
            while (isCurrent(connection, expectedGeneration)) {
                DoweSocketFrame frame = readFrame(input);
                if (frame.opcode == 8) {
                    handleClosed(connection, expectedGeneration, frame.payload);
                    return;
                }
                if (frame.opcode == 9) {
                    sendFrame(connection, 10, frame.payload);
                    continue;
                }
                if (frame.opcode == 10) continue;
                if (frame.opcode == 1) {
                    if (message != null) throw new java.io.IOException("Invalid WebSocket text frame");
                    message = new java.io.ByteArrayOutputStream();
                } else if (frame.opcode != 0 || message == null) {
                    continue;
                }
                if (message.size() + frame.payload.length > 1048576) throw new java.io.IOException("WebSocket message too large");
                message.write(frame.payload);
                if (frame.fin) {
                    String text = new String(message.toByteArray(), java.nio.charset.StandardCharsets.UTF_8);
                    message = null;
                    dispatch(null, onMessage, item("message", text));
                }
            }
        }

        private DoweSocketFrame readFrame(InputStream input) throws java.io.IOException {
            int first = readByte(input);
            int second = readByte(input);
            boolean fin = (first & 128) != 0;
            int opcode = first & 15;
            boolean masked = (second & 128) != 0;
            long length = second & 127;
            if (length == 126) {
                length = ((long) readByte(input) << 8) | readByte(input);
            } else if (length == 127) {
                length = 0;
                for (int index = 0; index < 8; index++) length = (length << 8) | readByte(input);
                if (length < 0) throw new java.io.IOException("Invalid WebSocket frame length");
            }
            if (length > 1048576) throw new java.io.IOException("WebSocket frame too large");
            byte[] mask = masked ? readBytes(input, 4) : null;
            byte[] payload = readBytes(input, (int) length);
            if (mask != null) {
                for (int index = 0; index < payload.length; index++) payload[index] = (byte) (payload[index] ^ mask[index & 3]);
            }
            return new DoweSocketFrame(fin, opcode, payload);
        }

        private int readByte(InputStream input) throws java.io.IOException {
            int value = input.read();
            if (value < 0) throw new java.io.IOException("WebSocket closed");
            return value;
        }

        private byte[] readBytes(InputStream input, int length) throws java.io.IOException {
            byte[] result = new byte[length];
            int offset = 0;
            while (offset < length) {
                int count = input.read(result, offset, length - offset);
                if (count < 0) throw new java.io.IOException("WebSocket closed");
                if (count == 0) continue;
                offset += count;
            }
            return result;
        }

        private void sendFrame(Socket connection, int opcode, byte[] payload) throws java.io.IOException {
            synchronized (this) {
                if (stopped || transport != connection || output == null) return;
                writeFrame(output, opcode, payload);
                output.flush();
            }
        }

        private void writeFrame(java.io.OutputStream stream, int opcode, byte[] payload) throws java.io.IOException {
            if (payload.length > 1048576) throw new java.io.IOException("WebSocket frame too large");
            byte[] mask = new byte[4];
            random.nextBytes(mask);
            stream.write(128 | (opcode & 15));
            if (payload.length < 126) {
                stream.write(128 | payload.length);
            } else if (payload.length <= 65535) {
                stream.write(254);
                stream.write((payload.length >>> 8) & 255);
                stream.write(payload.length & 255);
            } else {
                stream.write(255);
                long length = payload.length;
                for (int index = 7; index >= 0; index--) stream.write((int) (length >>> (index * 8)) & 255);
            }
            stream.write(mask);
            for (int index = 0; index < payload.length; index++) stream.write(payload[index] ^ mask[index & 3]);
        }

        private void handleClosed(Socket connection, long expectedGeneration, byte[] payload) {
            if (!isCurrent(connection, expectedGeneration)) return;
            try {
                sendFrame(connection, 8, payload);
            } catch (Exception error) {
                handleFailure(connection, expectedGeneration, error);
                return;
            }
            clearTransport(connection);
            dispatch("closed", onClose, item("close", closeData(payload)));
            scheduleReconnect(expectedGeneration);
        }

        private Map<String, Object> closeData(byte[] payload) {
            int code = payload.length >= 2 ? ((payload[0] & 255) << 8) | (payload[1] & 255) : 1000;
            String reason = payload.length > 2
                ? new String(payload, 2, payload.length - 2, java.nio.charset.StandardCharsets.UTF_8)
                : "";
            return doweObject("code", code, "reason", reason);
        }

        private void handleFailure(Socket connection, long expectedGeneration, Exception error) {
            synchronized (this) {
                if (stopped || generation != expectedGeneration) return;
                if (connection == null && transport != null) return;
                if (connection != null && transport != null && transport != connection) return;
                if (transport == connection) {
                    transport = null;
                    output = null;
                }
            }
            String message = error.getMessage() == null ? "WebSocket failure" : error.getMessage();
            dispatch("error", onError, item("error", message));
            scheduleReconnect(expectedGeneration);
        }

        private void scheduleReconnect(long expectedGeneration) {
            synchronized (this) {
                if (!reconnect || stopped || generation != expectedGeneration || currentUrl == null || reconnectTask != null) return;
                reconnectTask = () -> {
                    synchronized (this) { reconnectTask = null; }
                    connect();
                };
                handler.postDelayed(reconnectTask, Math.max(100, Math.min(60000, reconnectDelay)));
            }
        }

        private void sendCurrent() {
            if (sendPath == null) return;
            Object value = value(sendPath);
            if (value == null) return;
            String payload;
            try { payload = value instanceof String ? String.valueOf(value) : doweJson(value).toString(); }
            catch (Exception error) { return; }
            Socket connection;
            long expectedGeneration;
            Exception failure = null;
            synchronized (this) {
                connection = transport;
                expectedGeneration = generation;
                if (stopped || connection == null || output == null || payload.equals(lastSent)) return;
                try {
                    writeFrame(output, 1, payload.getBytes(java.nio.charset.StandardCharsets.UTF_8));
                    output.flush();
                    lastSent = payload;
                } catch (Exception error) {
                    failure = error;
                }
            }
            if (failure != null) handleFailure(connection, expectedGeneration, failure);
        }

        void stop() {
            Socket connection;
            synchronized (this) {
                stopped = true;
                generation++;
                if (reconnectTask != null) handler.removeCallbacks(reconnectTask);
                reconnectTask = null;
                connection = transport;
                transport = null;
                output = null;
                currentUrl = null;
                lastSent = null;
            }
            closeQuietly(connection);
        }

        private void clearTransport(Socket connection) {
            if (connection == null) return;
            synchronized (this) {
                if (transport == connection) {
                    transport = null;
                    output = null;
                }
            }
        }

        private void closeQuietly(Socket connection) {
            if (connection == null) return;
            try { connection.close(); } catch (Exception ignored) { }
        }

        private final class DoweSocketFrame {
            final boolean fin;
            final int opcode;
            final byte[] payload;

            DoweSocketFrame(boolean fin, int opcode, byte[] payload) {
                this.fin = fin;
                this.opcode = opcode;
                this.payload = payload;
            }
        }
    }
"#
}
