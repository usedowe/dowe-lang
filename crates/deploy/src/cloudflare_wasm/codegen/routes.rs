use super::super::*;
use super::helpers::memory_arg;
use wasm_encoder::{BlockType, Function, InstructionSink, ValType};

pub(super) fn handle_function(
    plans: &[EndpointPlan],
    not_found: Blob,
    invalid_json: Blob,
    max_dynamic_params: usize,
) -> Function {
    let local_count = 4 + max_dynamic_params.saturating_mul(2) as u32;
    let mut function = Function::new([(local_count, ValType::I32)]);
    let mut instructions = function.instructions();
    set_response(&mut instructions, 404, BodyKind::Text);
    instructions
        .i32_const(OUTPUT_BUFFER as i32)
        .local_set(OUTPUT_LOCAL);
    for plan in plans {
        emit_method_match(&mut instructions, plan.method);
        emit_route_match(&mut instructions, &plan.segments);
        instructions.i32_and().if_(BlockType::Empty);
        emit_response(&mut instructions, &plan.response, invalid_json);
        instructions.end();
    }
    encode_response(&mut instructions, not_found.pointer, not_found.length);
    instructions.end();
    function
}

pub(super) fn emit_method_match(instructions: &mut InstructionSink<'_>, method: Blob) {
    instructions
        .local_get(0)
        .local_get(1)
        .i32_const(method.pointer as i32)
        .i32_const(method.length as i32)
        .call(BYTES_EQUAL_FUNCTION);
}

pub(super) fn emit_route_match(instructions: &mut InstructionSink<'_>, segments: &[RouteSegment]) {
    instructions.block(BlockType::Result(ValType::I32));
    if segments.is_empty() {
        instructions.local_get(3).i32_const(1).i32_ne();
        emit_route_failure(instructions);
        instructions.i32_const(1).end();
        return;
    }
    instructions.local_get(3).i32_const(1).i32_lt_u();
    emit_route_failure(instructions);
    instructions
        .local_get(2)
        .i32_load8_u(memory_arg())
        .i32_const(b'/' as i32)
        .i32_ne();
    emit_route_failure(instructions);
    instructions.i32_const(1).local_set(PATH_CURSOR_LOCAL);

    for (index, segment) in segments.iter().enumerate() {
        instructions
            .local_get(2)
            .local_get(3)
            .local_get(PATH_CURSOR_LOCAL)
            .call(FIND_SLASH_FUNCTION)
            .local_set(SEGMENT_END_LOCAL);
        match segment {
            RouteSegment::Static(blob) => {
                instructions
                    .local_get(SEGMENT_END_LOCAL)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_sub()
                    .i32_const(blob.length as i32)
                    .i32_ne();
                emit_route_failure(instructions);
                instructions
                    .local_get(2)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_add()
                    .local_get(SEGMENT_END_LOCAL)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_sub()
                    .i32_const(blob.pointer as i32)
                    .i32_const(blob.length as i32)
                    .call(BYTES_EQUAL_FUNCTION)
                    .i32_eqz();
                emit_route_failure(instructions);
            }
            RouteSegment::Parameter { .. } => {
                instructions
                    .local_get(SEGMENT_END_LOCAL)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_eq();
                emit_route_failure(instructions);
                let parameter_index = segments[..index]
                    .iter()
                    .filter(|item| matches!(item, RouteSegment::Parameter { .. }))
                    .count() as u32;
                let start_local = PARAM_LOCALS_BASE + parameter_index * 2;
                instructions
                    .local_get(2)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_add()
                    .local_set(start_local)
                    .local_get(SEGMENT_END_LOCAL)
                    .local_get(PATH_CURSOR_LOCAL)
                    .i32_sub()
                    .local_set(start_local + 1);
            }
        }
        if index + 1 == segments.len() {
            instructions
                .local_get(SEGMENT_END_LOCAL)
                .local_get(3)
                .i32_ne();
            emit_route_failure(instructions);
        } else {
            instructions
                .local_get(SEGMENT_END_LOCAL)
                .local_get(3)
                .i32_eq();
            emit_route_failure(instructions);
            instructions
                .local_get(SEGMENT_END_LOCAL)
                .i32_const(1)
                .i32_add()
                .local_set(PATH_CURSOR_LOCAL);
        }
    }
    instructions.i32_const(1).end();
}

pub(super) fn emit_route_failure(instructions: &mut InstructionSink<'_>) {
    instructions.if_(BlockType::Empty).i32_const(0).br(1).end();
}

pub(super) fn emit_response(
    instructions: &mut InstructionSink<'_>,
    response: &ResponsePlan,
    invalid_json: Blob,
) {
    match response {
        ResponsePlan::Static(blob, kind) => {
            set_response(instructions, 200, *kind);
            encode_response(instructions, blob.pointer, blob.length);
        }
        ResponsePlan::Template(parts) => {
            set_response(instructions, 200, BodyKind::Text);
            instructions
                .i32_const(OUTPUT_BUFFER as i32)
                .local_set(OUTPUT_LOCAL);
            for part in parts {
                match part {
                    TemplatePart::Literal(blob) => emit_copy_literal(instructions, *blob),
                    TemplatePart::Parameter(index) => {
                        let start_local = PARAM_LOCALS_BASE + (*index as u32) * 2;
                        instructions
                            .local_get(OUTPUT_LOCAL)
                            .local_get(start_local)
                            .local_get(start_local + 1)
                            .call(COPY_BYTES_FUNCTION)
                            .local_get(OUTPUT_LOCAL)
                            .local_get(start_local + 1)
                            .i32_add()
                            .local_set(OUTPUT_LOCAL);
                    }
                }
            }
            encode_dynamic_response(instructions);
        }
        ResponsePlan::Greeting {
            prefix,
            suffix,
            parameter_index,
        } => {
            set_response(instructions, 200, BodyKind::Text);
            instructions
                .i32_const(OUTPUT_BUFFER as i32)
                .local_set(OUTPUT_LOCAL);
            emit_copy_literal(instructions, *prefix);
            if let Some(parameter_index) = parameter_index {
                let start_local = PARAM_LOCALS_BASE + (*parameter_index as u32) * 2;
                instructions
                    .local_get(OUTPUT_LOCAL)
                    .local_get(start_local)
                    .local_get(start_local + 1)
                    .call(COPY_BYTES_FUNCTION)
                    .local_get(OUTPUT_LOCAL)
                    .local_get(start_local + 1)
                    .i32_add()
                    .local_set(OUTPUT_LOCAL);
            }
            emit_copy_literal(instructions, *suffix);
            encode_dynamic_response(instructions);
        }
        ResponsePlan::CreatedJson => {
            set_response(instructions, 200, BodyKind::Json);
            instructions
                .local_get(4)
                .local_get(5)
                .i32_const(OUTPUT_BUFFER as i32)
                .call(RENDER_CREATED_JSON_FUNCTION)
                .local_set(RENDER_LENGTH_LOCAL)
                .local_get(RENDER_LENGTH_LOCAL)
                .i32_const(0)
                .i32_lt_s()
                .if_(BlockType::Empty)
                .i32_const(400)
                .global_set(0)
                .i32_const(BodyKind::Text as i32)
                .global_set(1)
                .i64_const(((invalid_json.pointer as i64) << 32) | invalid_json.length as i64)
                .return_()
                .end();
            encode_response_from_length(instructions, OUTPUT_BUFFER, RENDER_LENGTH_LOCAL);
        }
    }
}

pub(super) fn emit_copy_literal(instructions: &mut InstructionSink<'_>, blob: Blob) {
    if blob.length == 0 {
        return;
    }
    instructions
        .local_get(OUTPUT_LOCAL)
        .i32_const(blob.pointer as i32)
        .i32_const(blob.length as i32)
        .call(COPY_BYTES_FUNCTION)
        .local_get(OUTPUT_LOCAL)
        .i32_const(blob.length as i32)
        .i32_add()
        .local_set(OUTPUT_LOCAL);
}

pub(super) fn set_response(instructions: &mut InstructionSink<'_>, status: i32, kind: BodyKind) {
    instructions
        .i32_const(status)
        .global_set(0)
        .i32_const(kind as i32)
        .global_set(1);
}

pub(super) fn encode_response(instructions: &mut InstructionSink<'_>, pointer: u32, length: u32) {
    instructions
        .i64_const(((pointer as i64) << 32) | length as i64)
        .return_();
}

pub(super) fn encode_dynamic_response(instructions: &mut InstructionSink<'_>) {
    instructions
        .i64_const((OUTPUT_BUFFER as i64) << 32)
        .local_get(OUTPUT_LOCAL)
        .i32_const(OUTPUT_BUFFER as i32)
        .i32_sub()
        .i64_extend_i32_u()
        .i64_or()
        .return_();
}

pub(super) fn encode_response_from_length(
    instructions: &mut InstructionSink<'_>,
    pointer: u32,
    length_local: u32,
) {
    instructions
        .i64_const((pointer as i64) << 32)
        .local_get(length_local)
        .i64_extend_i32_u()
        .i64_or()
        .return_();
}
