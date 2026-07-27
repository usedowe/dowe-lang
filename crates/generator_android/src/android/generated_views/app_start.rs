fn android_runtime_app_start() -> &'static str {
    r#"private data class DoweRouteEntry(val path: String, val fragment: String?)

@Composable
fun DoweApp(startPath: String = DoweRoutes.initialPath, startFragment: String? = null, navigationRequest: Int = 0) {
"#
}
