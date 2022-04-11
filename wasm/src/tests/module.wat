;; compile with wat2wasm wasm/src/tests/module.wat -o wasm/src/tests/module.wasm

(module 
    (type
        (func (param i32) (result i32) )
    )
    (func (type 0)
        (local i32)
        
        local.get 0
        i32.const 42
        i32.add
        
        return
    )
)