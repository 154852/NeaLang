const primaryPath = process.argv[2];
// const stdPath = process.argv[3];

const fs = require("fs");

class Heap {
    constructor(offset) {
        this.offset = offset;
    }

    alloc(size) {
        this.offset += size;
        return this.offset - size;
    }

    free(addr, size) {
        if (addr + size == this.offset) {
            this.offset -= size;
        }
    }
}

(async function() {
    let mem;
    let heap;

    const core = {
        exit: (code) => process.exit(code),
        putchar: (char) => process.stdout.write(String.fromCharCode(char)),
        
        new_object: (size) => heap.alloc(size),
        new_slice: (length, size) => {
            let addr = heap.alloc(8);
            let data = heap.alloc(length * size);
            let sliced = new Uint8Array(mem.buffer);
            
            sliced[addr] = data & 0xff;
            sliced[addr + 1] = (data >> 8) & 0xff;
            sliced[addr + 2] = (data >> 16) & 0xff;
            sliced[addr + 3] = (data >> 24) & 0xff;

            sliced[addr + 4] = length & 0xff;
            sliced[addr + 4 + 1] = (length >> 8) & 0xff;
            sliced[addr + 4 + 2] = (length >> 16) & 0xff;
            sliced[addr + 4 + 3] = (length >> 24) & 0xff;

            return addr;
        },

        drop_object: (addr, size) => heap.free(addr, size),
        drop_slice: (slice, size) => {
            let sliced = new Uint8Array(mem.buffer);
            let data_addr = sliced[slice] | (sliced[slice + 1] << 8) | (sliced[slice + 2] << 16) | (sliced[slice + 3] << 24);
            let slice_length = sliced[slice + 4] | (sliced[slice + 4 + 1] << 8) | (sliced[slice + 4 + 2] << 16) | (sliced[slice + 4 + 3] << 24);

            heap.free(data_addr, slice_length * size);
            heap.free(slice, 8);
        }
    };
    
    let primary = await WebAssembly.instantiate(fs.readFileSync(primaryPath), {
        core
    });

    mem = primary.instance.exports.mem;
    heap = new Heap(primary.instance.exports.mem_size + 0);
    primary.instance.exports.main();
})();