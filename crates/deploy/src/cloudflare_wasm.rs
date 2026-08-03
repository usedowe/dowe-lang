use dowe_compiler::{Endpoint, EndpointBehavior, HttpMethod};
mod codegen;

const DATA_BASE: u32 = 1024;
const OUTPUT_BUFFER: u32 = 262144;
const MEMORY_PAGES: u64 = 8;
const BYTES_EQUAL_FUNCTION: u32 = 0;
const FIND_SLASH_FUNCTION: u32 = 1;
const COPY_BYTES_FUNCTION: u32 = 2;
const RENDER_CREATED_JSON_FUNCTION: u32 = 3;
const HANDLE_FUNCTION: u32 = 4;
const OUTPUT_LOCAL: u32 = 6;
const PATH_CURSOR_LOCAL: u32 = 7;
const SEGMENT_END_LOCAL: u32 = 8;
const RENDER_LENGTH_LOCAL: u32 = 9;
const PARAM_LOCALS_BASE: u32 = 10;

#[derive(Clone, Copy)]
struct Blob {
    pointer: u32,
    length: u32,
}

struct DataStore {
    next: u32,
    segments: Vec<(u32, Vec<u8>)>,
}

impl DataStore {
    fn new() -> Self {
        Self {
            next: DATA_BASE,
            segments: Vec::new(),
        }
    }

    fn add_bytes(&mut self, bytes: &[u8]) -> Blob {
        let pointer = self.next;
        let length = bytes.len() as u32;
        self.next = self.next.saturating_add(length).saturating_add(1);
        self.segments.push((pointer, bytes.to_vec()));
        Blob { pointer, length }
    }

    fn add_text(&mut self, value: &str) -> Blob {
        self.add_bytes(value.as_bytes())
    }
}

struct EndpointPlan {
    method: Blob,
    segments: Vec<RouteSegment>,
    response: ResponsePlan,
    dynamic_params: usize,
}

enum RouteSegment {
    Static(Blob),
    Parameter { name: String },
}

enum ResponsePlan {
    Static(Blob, BodyKind),
    Template(Vec<TemplatePart>),
    Greeting {
        prefix: Blob,
        suffix: Blob,
        parameter_index: Option<usize>,
    },
    CreatedJson,
}

enum TemplatePart {
    Literal(Blob),
    Parameter(usize),
}

#[derive(Clone, Copy)]
enum BodyKind {
    Text = 0,
    Json = 1,
}

pub fn generate(endpoints: &[Endpoint]) -> Vec<u8> {
    let mut data = DataStore::new();
    let not_found = data.add_text("Not Found");
    let invalid_json = data.add_text("Expected JSON object");
    let created_prefix = data.add_text("{\"created\":true");
    let plans = endpoints
        .iter()
        .map(|endpoint| endpoint_plan(endpoint, &mut data))
        .collect::<Vec<_>>();
    let max_dynamic_params = plans
        .iter()
        .map(|plan| plan.dynamic_params)
        .max()
        .unwrap_or_default();
    codegen::encode(
        &data,
        &plans,
        not_found,
        invalid_json,
        created_prefix,
        max_dynamic_params,
    )
}

fn endpoint_plan(endpoint: &Endpoint, data: &mut DataStore) -> EndpointPlan {
    let segments = endpoint
        .path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if segment.starts_with(':') {
                RouteSegment::Parameter {
                    name: segment.trim_start_matches(':').to_string(),
                }
            } else {
                RouteSegment::Static(data.add_text(segment))
            }
        })
        .collect::<Vec<_>>();
    let dynamic_params = segments
        .iter()
        .filter(|segment| matches!(segment, RouteSegment::Parameter { .. }))
        .count();
    let response = match &endpoint.behavior {
        EndpointBehavior::StaticText(value) => {
            ResponsePlan::Static(data.add_text(value), BodyKind::Text)
        }
        EndpointBehavior::TextTemplate(value) => {
            ResponsePlan::Template(template_parts(value, &segments, data))
        }
        EndpointBehavior::UserGreeting => ResponsePlan::Greeting {
            prefix: data.add_text("Hello User "),
            suffix: data.add_text("!"),
            parameter_index: segments.iter().enumerate().find_map(|(index, segment)| {
                if !matches!(segment, RouteSegment::Parameter { .. }) {
                    return None;
                }
                Some(
                    segments[..index]
                        .iter()
                        .filter(|item| matches!(item, RouteSegment::Parameter { .. }))
                        .count(),
                )
            }),
        },
        EndpointBehavior::CreatePostJson => ResponsePlan::CreatedJson,
        EndpointBehavior::HttpProxy(_)
        | EndpointBehavior::HttpReverseProxy(_)
        | EndpointBehavior::HttpBytes(_)
        | EndpointBehavior::HttpActionJson(_)
        | EndpointBehavior::AgentResponse(_)
        | EndpointBehavior::StoreInsertJson(_)
        | EndpointBehavior::StoreQueryJson(_)
        | EndpointBehavior::StoreTransactionJson(_)
        | EndpointBehavior::StoreActionJson(_)
        | EndpointBehavior::KvActionJson(_)
        | EndpointBehavior::QueueActionJson(_)
        | EndpointBehavior::VectorActionJson(_) => ResponsePlan::Static(
            data.add_text("Unsupported Cloudflare route"),
            BodyKind::Text,
        ),
    };
    EndpointPlan {
        method: data.add_text(http_method(endpoint.method)),
        segments,
        response,
        dynamic_params,
    }
}

fn template_parts(
    value: &str,
    segments: &[RouteSegment],
    data: &mut DataStore,
) -> Vec<TemplatePart> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = value[cursor..].find("{req.params.") {
        let start = cursor + relative_start;
        if start > cursor {
            parts.push(TemplatePart::Literal(data.add_text(&value[cursor..start])));
        }
        let Some(relative_end) = value[start..].find('}') else {
            parts.push(TemplatePart::Literal(data.add_text(&value[start..])));
            cursor = value.len();
            break;
        };
        let end = start + relative_end + 1;
        let name = &value[start + "{req.params.".len()..end - 1];
        let parameter_index = segments.iter().enumerate().find_map(|(index, segment)| {
            let RouteSegment::Parameter {
                name: parameter_name,
            } = segment
            else {
                return None;
            };
            if parameter_name == name {
                Some(index)
            } else {
                None
            }
        });
        if let Some(parameter_index) = parameter_index {
            let dynamic_index = segments[..parameter_index]
                .iter()
                .filter(|segment| matches!(segment, RouteSegment::Parameter { .. }))
                .count();
            parts.push(TemplatePart::Parameter(dynamic_index));
        } else {
            parts.push(TemplatePart::Literal(data.add_text(&value[start..end])));
        }
        cursor = end;
    }
    if cursor < value.len() {
        parts.push(TemplatePart::Literal(data.add_text(&value[cursor..])));
    }
    if parts.is_empty() {
        parts.push(TemplatePart::Literal(data.add_text(value)));
    }
    parts
}

fn http_method(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}
