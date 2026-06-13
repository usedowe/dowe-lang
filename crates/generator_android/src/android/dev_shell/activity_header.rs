fn dev_activity_header() -> &'static str {
    r#"package dev.dowe.generated;

import android.app.Activity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.graphics.Color;
import android.graphics.Insets;
import android.graphics.Canvas;
import android.graphics.Matrix;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.text.SpannableString;
import android.text.style.ForegroundColorSpan;
import android.text.Editable;
import android.text.TextWatcher;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowInsets;
import android.webkit.WebView;
import android.widget.Button;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.HorizontalScrollView;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.MediaController;
import android.widget.ScrollView;
import android.widget.SeekBar;
import android.widget.PopupWindow;
import android.widget.TextView;
import android.widget.VideoView;
import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.function.Consumer;
import org.json.JSONArray;
import org.json.JSONObject;

@SuppressWarnings({"unchecked", "deprecation"})
public class DoweDevActivity extends Activity {
    private static final int DOWE_JUSTIFY_START = 0;
    private static final int DOWE_JUSTIFY_CENTER = 1;
    private static final int DOWE_JUSTIFY_END = 2;
    private static final int DOWE_JUSTIFY_BETWEEN = 3;
    private static final int DOWE_JUSTIFY_AROUND = 4;
    private static final int DOWE_JUSTIFY_EVENLY = 5;
    private static final int DOWE_ALIGN_START = 0;
    private static final int DOWE_ALIGN_CENTER = 1;
    private static final int DOWE_ALIGN_END = 2;
    private static final int DOWE_ALIGN_STRETCH = 3;
    private static final int DOWE_ALIGN_BASELINE = 4;
"#
}
