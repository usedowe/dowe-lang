use std::collections::HashMap;

pub(super) fn match_route(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_segments = pattern.trim_matches('/').split('/').collect::<Vec<_>>();
    let path_segments = path.trim_matches('/').split('/').collect::<Vec<_>>();

    if pattern == "/" && path == "/" {
        return Some(HashMap::new());
    }

    let mut params = HashMap::new();
    let splat = pattern_segments
        .last()
        .and_then(|segment| segment.strip_prefix('*'));
    let fixed_pattern_len = if splat.is_some() {
        pattern_segments.len().saturating_sub(1)
    } else {
        pattern_segments.len()
    };

    if let Some(splat_name) = splat {
        if splat_name.is_empty() || path_segments.len() <= fixed_pattern_len {
            return None;
        }
    } else if pattern_segments.len() != path_segments.len() {
        return None;
    }

    for (pattern_segment, path_segment) in pattern_segments
        .iter()
        .take(fixed_pattern_len)
        .zip(path_segments.iter())
    {
        if let Some(param_name) = pattern_segment.strip_prefix(':') {
            params.insert(param_name.to_string(), (*path_segment).to_string());
        } else if pattern_segment != path_segment {
            return None;
        }
    }

    if let Some(splat_name) = splat {
        params.insert(
            splat_name.to_string(),
            path_segments[fixed_pattern_len..].join("/"),
        );
    }

    Some(params)
}
