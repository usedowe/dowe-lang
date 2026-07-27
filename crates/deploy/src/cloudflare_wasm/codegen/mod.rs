mod helpers;
mod routes;

use super::*;
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ExportKind, ExportSection, FunctionSection, GlobalSection,
    GlobalType, MemorySection, MemoryType, Module, TypeSection, ValType,
};

pub(super) fn encode(
    data: &DataStore,
    plans: &[EndpointPlan],
    not_found: Blob,
    invalid_json: Blob,
    created_prefix: Blob,
    max_dynamic_params: usize,
) -> Vec<u8> {
    let mut types = TypeSection::new();
    types.ty().function(
        [ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        [ValType::I32],
    );
    types
        .ty()
        .function([ValType::I32, ValType::I32, ValType::I32], [ValType::I32]);
    types
        .ty()
        .function([ValType::I32, ValType::I32, ValType::I32], []);
    types
        .ty()
        .function([ValType::I32, ValType::I32, ValType::I32], [ValType::I32]);
    types.ty().function(
        [
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
        ],
        [ValType::I64],
    );

    let mut functions = FunctionSection::new();
    functions.function(0);
    functions.function(1);
    functions.function(2);
    functions.function(3);
    functions.function(4);

    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: MEMORY_PAGES,
        maximum: Some(MEMORY_PAGES),
        memory64: false,
        shared: false,
        page_size_log2: None,
    });

    let mut globals = GlobalSection::new();
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(404),
    );
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(BodyKind::Text as i32),
    );

    let mut exports = ExportSection::new();
    exports.export("memory", ExportKind::Memory, 0);
    exports.export("handle", ExportKind::Func, HANDLE_FUNCTION);
    exports.export("response_status", ExportKind::Global, 0);
    exports.export("response_kind", ExportKind::Global, 1);

    let mut code = CodeSection::new();
    code.function(&helpers::bytes_equal_function());
    code.function(&helpers::find_slash_function());
    code.function(&helpers::copy_bytes_function());
    code.function(&helpers::render_created_json_function(created_prefix));
    code.function(&routes::handle_function(
        plans,
        not_found,
        invalid_json,
        max_dynamic_params,
    ));

    let mut data_section = DataSection::new();
    for (pointer, bytes) in &data.segments {
        data_section.active(
            0,
            &ConstExpr::i32_const(*pointer as i32),
            bytes.iter().copied(),
        );
    }

    let mut module = Module::new();
    module
        .section(&types)
        .section(&functions)
        .section(&memory)
        .section(&globals)
        .section(&exports)
        .section(&code)
        .section(&data_section);
    module.finish()
}
