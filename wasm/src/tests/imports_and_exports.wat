;; compile with wat2wasm wasm/src/tests/imports_and_exports.wat -o wasm/src/tests/imports_and_exports.wasm

(module 
    (type (func (param i32) (param i32) (result i32)) )
    (type (func (param i32) (result i32)) )
    (import "std" "add"
        (func (type 0))
    )
    (export "add_12"
        (func 1)
    )
    (func (type 1)
        (local i32)
        
        local.get 0
        i32.const 12
        call 0
        
        return
    )
)