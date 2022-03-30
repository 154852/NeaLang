const primaryPath = process.argv[2];

const fs = require("fs");

const ALLOC_SIZE_OFF = 0;
const ALLOC_HEADER_SIZE = 4;
const FREE_FD_OFF = ALLOC_HEADER_SIZE;
const FREE_FD_NONE = 0xffffffff;
const FREE_SELF_SIZE = 4;

class NLProcessMemory {
    constructor(memory, heap_start) {
        this.memory = memory;
        this.u8mem = new Uint8Array(memory.buffer);

        // 8 byte align the heap
        this.heap_start = heap_start & 7 == 0? heap_start:(heap_start + 16 - (heap_start & 7));
        this.heap_end = this.heap_start;

        // Freelist buckets, of 4, 8, 12, 16, 20, 24, 28 and 32 byte blocks - the most common sizes for structures
        // Addresses refer to the beginning of the block, not the FD pointer as in GLIBC
        this.firstfree = {
            4: null,
            8: null,
            12: null,
            16: null,
            20: null,
            24: null,
            28: null,
            32: null,
        };
        this.any_firstfree = null;
    }

    read_u32(addr) {
        // It would be nice to align addresses and thus use a Uint32Array, but we can't every guarantee alignment
        return this.u8mem[addr] | (this.u8mem[addr + 1] << 8) | (this.u8mem[addr + 2] << 16) | (this.u8mem[addr + 3] << 24);
    }

    write_u32(addr, value) {
        this.u8mem[addr] = value & 0xff;
        this.u8mem[addr + 1] = (value >> 8) & 0xff;
        this.u8mem[addr + 2] = (value >> 16) & 0xff;
        this.u8mem[addr + 3] = (value >> 24) & 0xff;
    }

    // Size is already assumed to be aligned
    allocate_from_wilderness(size) {
        const addr = this.heap_end;
        this.heap_end += ALLOC_HEADER_SIZE + size;

        this.write_u32(addr + ALLOC_SIZE_OFF, size);

        // + 4 to get the usable region, addr only points to the start of the block
        return addr + ALLOC_HEADER_SIZE;
    }

    heap_allocate(size) {
        if (size & 3) size += 4 - (size & 3);
        if (size < 4) size = FREE_SELF_SIZE; // make sure the free data can fit
        
        if (size in this.firstfree) {
            // This is probably sub-optimal - we should likely look at other buckets, bigger than ours first.
            // Imagine first 1000 4 byte blocks are allocated, then freed, then 500 8 byte blocks. The 4 byte blocks
            // will not be used, but they probably should be.
            if (this.firstfree[size] == null) {
                return this.allocate_from_wilderness(size);
            }
            
            let addr = this.firstfree[size];
            let fd = this.read_u32(addr + FREE_FD_OFF);
            this.firstfree[addr] = fd == FREE_FD_NONE? null:fd;

            // Alloc block should already be set up
            return addr + ALLOC_HEADER_SIZE;
        }

        let firstfree = this.any_firstfree;
        let firstfreeprev = null;
        while (firstfree != null) {
            let free_size = this.read_u32(firstfree + ALLOC_SIZE_OFF);
            let fd = this.read_u32(firstfree + FREE_FD_OFF);

            if (size > free_size) {
                firstfreeprev = firstfree;
                firstfree = fd == FREE_FD_NONE? null:fd;
                continue;
            }

            if (free_size - size >= ALLOC_HEADER_SIZE + FREE_SELF_SIZE + 4) { // Split the block
                const block_a_full_size = ALLOC_HEADER_SIZE + size;
                const block_a_content_size = size;
                const block_a_addr = firstfree;

                const block_b_full_size = free_size - size;
                const block_b_content_size = block_b_full_size - ALLOC_HEADER_SIZE;
                const block_b_addr = firstfree + block_a_full_size;

                this.write_u32(block_a_addr + ALLOC_SIZE_OFF, block_a_content_size);
                this.write_u32(block_b_addr + ALLOC_SIZE_OFF, block_b_content_size);

                // Effectively free block b
                if (block_b_content_size in this.firstfree) {
                    this.write_u32(block_b_addr + FREE_FD_OFF, this.firstfree[block_b_content_size] ?? FREE_FD_NONE);
                    this.firstfree[block_b_content_size] = block_b_addr;
                } else {
                    this.write_u32(block_b_addr + FREE_FD_OFF, this.any_firstfree ?? FREE_FD_NONE);
                    this.any_firstfree = block_b_addr;
                }

                // A -> [B] -> C  becomes  A -> C
                if (firstfreeprev == null)
                    this.any_firstfree = fd == FREE_FD_NONE? null:fd;
                else
                    this.write_u32(firstfreeprev + FREE_FD_OFF, fd);

                return block_a_addr + ALLOC_HEADER_SIZE;
            } else { // Keep the block whole
                // A -> [B] -> C  becomes  A -> C
                if (firstfreeprev == null)
                    this.any_firstfree = fd == FREE_FD_NONE? null:fd;
                else
                    this.write_u32(firstfreeprev + FREE_FD_OFF, fd);
                
                return firstfree + ALLOC_HEADER_SIZE;
            }
        }

        // No free block found small enough
        return this.allocate_from_wilderness(size);
    }

    heap_free(addr) {
        addr -= ALLOC_HEADER_SIZE;

        let size = this.read_u32(addr + ALLOC_SIZE_OFF);

        if (size in this.firstfree) {
            this.write_u32(addr + FREE_FD_OFF, this.firstfree[size] ?? FREE_FD_NONE);
            this.firstfree[size] = addr;
        } else {
            this.write_u32(addr + FREE_FD_OFF, this.any_firstfree ?? FREE_FD_NONE);
            this.any_firstfree = addr;
        }
    }
}

(async function() {
    let mem_ctx;

    const core = {
        exit: (code) => process.exit(code),
        putchar: (char) => process.stdout.write(String.fromCharCode(char)),
        
        new_object: (size) => mem_ctx.heap_allocate(size),
        new_slice: (length, size) => {
            let addr = mem_ctx.heap_allocate(8);
            let data = mem_ctx.heap_free(length * size);
            
            mem_ctx.write_u32(addr, data);
            mem_ctx.write_u32(addr + 4, length);

            return addr;
        },

        drop_object: (addr, size) => mem_ctx.heap_free(addr, size),
        drop_slice: (slice, size) => {
            let data_addr = mem_ctx.read_u32(slice);
            let slice_length = mem_ctx.read_u32(slice + 4);

            mem_ctx.heap_free(data_addr, slice_length * size);
            mem_ctx.heap_free(slice, 8);
        }
    };
    
    let primary = await WebAssembly.instantiate(fs.readFileSync(primaryPath), {
        core
    });

    mem_ctx = new NLProcessMemory(primary.instance.exports.mem, primary.instance.exports.mem_size);
    primary.instance.exports.main();
})();