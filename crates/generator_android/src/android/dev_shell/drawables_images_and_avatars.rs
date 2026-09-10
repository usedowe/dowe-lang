r#"    private FrameLayout doweImage(String source, String alt, String aspect, String objectFit, int backgroundColor, Integer borderColor) {
        DoweImageLayout view = new DoweImageLayout(this, doweImageAspect(aspect));
        GradientDrawable loadedBackground = borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS);
        view.setBackground(borderColor == null ? doweBackground(DOWE_SURFACE, DOWE_RADIUS) : doweInputBackground(DOWE_SURFACE, borderColor, DOWE_RADIUS));
        ImageView image = new ImageView(this);
        image.setContentDescription(alt.isEmpty() ? null : alt);
        image.setScaleType(doweImageScaleType(objectFit));
        view.addView(image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        new Thread(() -> {
            Bitmap bitmap = doweLoadImageBitmap(source);
            if (bitmap != null) {
                runOnUiThread(() -> {
                    image.setImageBitmap(bitmap);
                    view.setBackground(loadedBackground);
                });
            }
        }).start();
        return view;
    }

    private int doweAvatarSize(String size) {
        if ("xs".equals(size)) return 24;
        if ("sm".equals(size)) return 32;
        if ("md".equals(size)) return 40;
        if ("lg".equals(size)) return 48;
        if ("xl".equals(size)) return 64;
        if ("2xl".equals(size)) return 80;
        if ("3xl".equals(size)) return 96;
        if ("4xl".equals(size)) return 112;
        if ("5xl".equals(size)) return 128;
        if ("6xl".equals(size)) return 144;
        if ("7xl".equals(size)) return 160;
        return 40;
    }

    private float doweAvatarTextSize(String size) {
        if ("xs".equals(size)) return 12f;
        if ("sm".equals(size)) return 14f;
        if ("md".equals(size)) return 16f;
        if ("lg".equals(size)) return 18f;
        if ("xl".equals(size)) return 24f;
        if ("2xl".equals(size)) return 28f;
        if ("3xl".equals(size)) return 32f;
        if ("4xl".equals(size)) return 36f;
        if ("5xl".equals(size)) return 40f;
        if ("6xl".equals(size)) return 44f;
        if ("7xl".equals(size)) return 48f;
        return 16f;
    }

    private FrameLayout doweAvatarImage(String source, String alt, String fallback, int backgroundColor, int contentColor, float textSize, String font) {
        FrameLayout view = new FrameLayout(this);
        view.setBackground(doweBackground(backgroundColor, 999f));
        TextView initials = doweText(fallback, contentColor, textSize, 600, 0f, 1.2f, font);
        initials.setGravity(Gravity.CENTER);
        view.addView(initials, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        ImageView image = new ImageView(this);
        image.setContentDescription(alt.isEmpty() ? null : alt);
        image.setScaleType(ImageView.ScaleType.CENTER_CROP);
        view.addView(image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        new Thread(() -> {
            Bitmap bitmap = doweLoadImageBitmap(source);
            if (bitmap != null) {
                runOnUiThread(() -> image.setImageBitmap(bitmap));
            }
        }).start();
        return view;
    }

    private LinearLayout doweAvatarGroup(String dataPath, String[] sources, String[] names, String[] alts, int size, int textSize, int maxCount, boolean inline, boolean bordered, int backgroundColor, int contentColor, int borderColor, String font) {
        LinearLayout group = doweContainer(true);
        group.setGravity(Gravity.CENTER_VERTICAL);
        ArrayList<Map<String, Object>> rows = dataPath == null ? new ArrayList<>() : doweRows(dataPath);
        int total = rows.isEmpty() ? sources.length : rows.size();
        int visible = maxCount > 0 ? Math.min(maxCount, total) : total;
        for (int index = 0; index < visible; index++) {
            String source;
            String name;
            String alt;
            if (rows.isEmpty()) {
                source = index < sources.length ? sources[index] : "";
                name = index < names.length ? names[index] : "";
                alt = index < alts.length ? alts[index] : "";
            } else {
                Map<String, Object> row = rows.get(index);
                source = doweTextValue("item.src", row);
                name = doweTextValue("item.name", row);
                alt = doweTextValue("item.alt", row);
            }
            String identity = name.isEmpty() ? alt : name;
            String fallback = identity.isEmpty() ? "A" : identity.substring(0, 1).toUpperCase(java.util.Locale.ROOT);
            FrameLayout avatar;
            if (source.isEmpty()) {
                avatar = new FrameLayout(this);
                TextView initials = doweText(fallback, contentColor, textSize, 600, 0f, 1.2f, font);
                initials.setGravity(Gravity.CENTER);
                avatar.addView(initials, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            } else {
                avatar = doweAvatarImage(source, alt.isEmpty() ? identity : alt, fallback, backgroundColor, contentColor, textSize, font);
            }
            avatar.setBackground(doweStyledBackground(backgroundColor, bordered ? borderColor : null, bordered ? 3 : null, 999f));
            doweRound(avatar, 999f);
            LinearLayout.LayoutParams avatarParams = new LinearLayout.LayoutParams(doweDp(size), doweDp(size));
            if (index > 0) {
                avatarParams.setMargins(doweDp(inline ? 8 : -12), 0, 0, 0);
            }
            group.addView(avatar, avatarParams);
        }
        int hiddenCount = Math.max(0, total - visible);
        if (hiddenCount > 0) {
            TextView counter = doweText("+" + hiddenCount, contentColor, textSize, 600, 0f, 1.2f, font);
            counter.setGravity(Gravity.CENTER);
            counter.setBackground(doweStyledBackground(backgroundColor, bordered ? borderColor : null, bordered ? 3 : 1, 999f));
            doweRound(counter, 999f);
            LinearLayout.LayoutParams counterParams = new LinearLayout.LayoutParams(doweDp(size), doweDp(size));
            if (visible > 0) {
                counterParams.setMargins(doweDp(inline ? 8 : -12), 0, 0, 0);
            }
            group.addView(counter, counterParams);
        }
        return group;
    }

    private Bitmap doweLoadImageBitmap(String source) {
        Bitmap cached = doweImageMemoryCache.get(source);
        if (cached != null) {
            return cached;
        }
        Object lock = doweImageLoadLocks.computeIfAbsent(source, value -> new Object());
        try {
            synchronized (lock) {
                cached = doweImageMemoryCache.get(source);
                if (cached != null) {
                    return cached;
                }
                Bitmap bitmap = doweReadImageBitmap(source);
                if (bitmap != null) {
                    doweImageMemoryCache.put(source, bitmap);
                }
                return bitmap;
            }
        } finally {
            doweImageLoadLocks.remove(source, lock);
        }
    }

    private Bitmap doweReadImageBitmap(String source) {
        try {
            Bitmap bitmap;
            if (source.startsWith("data:image/")) {
                int separator = source.indexOf(',');
                if (separator < 0) return null;
                byte[] bytes = Base64.decode(source.substring(separator + 1), Base64.DEFAULT);
                bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.length);
            } else if (source.startsWith("https://") || source.startsWith("http://")) {
                File directory = new File(getCacheDir(), "dowe-images");
                directory.mkdirs();
                File file = new File(directory, doweImageCacheKey(source));
                if (file.isFile()) {
                    bitmap = BitmapFactory.decodeFile(file.getAbsolutePath());
                    if (bitmap != null) {
                        file.setLastModified(System.currentTimeMillis());
                        doweImageMemoryCache.put(source, bitmap);
                        return bitmap;
                    }
                    file.delete();
                }
                File temporary = new File(directory, file.getName() + ".tmp");
                HttpURLConnection connection = (HttpURLConnection) new URL(source).openConnection();
                connection.setConnectTimeout(10000);
                connection.setReadTimeout(10000);
                connection.setUseCaches(true);
                connection.setInstanceFollowRedirects(true);
                connection.setRequestProperty("User-Agent", "Dowe/1.0");
                connection.setRequestProperty("Accept", "image/*");
                try {
                    if (connection.getResponseCode() < 200 || connection.getResponseCode() >= 300) {
                        return null;
                    }
                    java.io.ByteArrayOutputStream bytes = new java.io.ByteArrayOutputStream();
                    try (InputStream input = connection.getInputStream()) {
                        byte[] buffer = new byte[16384];
                        int count;
                        while ((count = input.read(buffer)) != -1) {
                            bytes.write(buffer, 0, count);
                        }
                    }
                    byte[] imageBytes = bytes.toByteArray();
                    bitmap = BitmapFactory.decodeByteArray(imageBytes, 0, imageBytes.length);
                    if (bitmap == null) {
                        return null;
                    }
                    try (FileOutputStream output = new FileOutputStream(temporary)) {
                        output.write(imageBytes);
                    }
                    temporary.renameTo(file);
                } finally {
                    connection.disconnect();
                    temporary.delete();
                    doweTrimImageDiskCache(directory);
                }
            } else {
                String assetPath = source.startsWith("/") ? source.substring(1) : source;
                if (assetPath.startsWith("assets/")) assetPath = assetPath.substring(7);
                try (InputStream input = getAssets().open(assetPath)) {
                    bitmap = BitmapFactory.decodeStream(input);
                }
            }
            return bitmap;
        } catch (Exception error) {
            return null;
        }
    }

    private String doweImageCacheKey(String source) throws Exception {
        byte[] bytes = MessageDigest.getInstance("SHA-256").digest(source.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        StringBuilder key = new StringBuilder();
        for (byte value : bytes) {
            key.append(String.format("%02x", value));
        }
        return key.toString();
    }

    private void doweTrimImageDiskCache(File directory) {
        File[] files = directory.listFiles();
        if (files == null) {
            return;
        }
        long total = 0L;
        for (File file : files) {
            total += file.length();
        }
        java.util.Arrays.sort(files, (left, right) -> Long.compare(left.lastModified(), right.lastModified()));
        for (File file : files) {
            if (total <= DOWE_IMAGE_DISK_CACHE_BYTES) {
                break;
            }
            long size = file.length();
            if (file.delete()) {
                total -= size;
            }
        }
    }

    private ImageView.ScaleType doweImageScaleType(String objectFit) {
        if ("contain".equals(objectFit)) {
            return ImageView.ScaleType.FIT_CENTER;
        }
        if ("fill".equals(objectFit)) {
            return ImageView.ScaleType.FIT_XY;
        }
        if ("none".equals(objectFit)) {
            return ImageView.ScaleType.CENTER;
        }
        return ImageView.ScaleType.CENTER_CROP;
    }

"#
