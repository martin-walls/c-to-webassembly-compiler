use crate::backend::wasm_indices::TypeIdx;
use crate::backend::wasm_instructions::WasmExpression;
use crate::backend::wasm_types::ValType;

pub struct WasmProgram {
    types: TypesSection,
    imports: ImportsSection,
    functions: FunctionsSection,
    tables: TablesSection,
    memory: MemorySection,
    globals: GlobalsSection,
    exports: ExportsSection,
    start: StartSection,
    element: ElementSection,
    code: CodeSection,
    data: DataSection,
}

impl WasmProgram {
    pub fn new() -> Self {
        WasmProgram {
            types: TypesSection::new(),
            imports: ImportsSection::new(),
            functions: FunctionsSection::new(),
            tables: TablesSection::new(),
            memory: MemorySection::new(),
            globals: GlobalsSection::new(),
            exports: ExportsSection::new(),
            start: StartSection::new(),
            element: ElementSection::new(),
            code: CodeSection::new(),
            data: DataSection::new(),
        }
    }
}

pub struct TypesSection {}

impl TypesSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct ImportsSection {}

impl ImportsSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct FunctionsSection {}

impl FunctionsSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct TablesSection {}

impl TablesSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct MemorySection {}

impl MemorySection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct GlobalsSection {}

impl GlobalsSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct ExportsSection {}

impl ExportsSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct StartSection {}

impl StartSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct ElementSection {}

impl ElementSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct CodeSection {}

impl CodeSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct DataSection {}

impl DataSection {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct WasmFunction {
    type_idx: TypeIdx,
    local_declarations: Vec<ValType>,
    body: WasmExpression,
}
