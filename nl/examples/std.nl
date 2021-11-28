func [arch="x86,wasm"] exit(code: i32) extern
func [arch="x86,wasm"] putchar(chr: u32) extern
func [arch="x86,wasm", alloc] nl_new_object(size: uptr): uptr extern
func [arch="x86,wasm", alloc_slice] nl_new_slice(count: uptr, size: uptr): uptr extern

func [arch="java", location="nl/examples/Std"] exit(code: i32) extern
func [arch="java", location="nl/examples/Std"] putchar(b: u32) extern

struct String {
	data: u8[]
}

func String.len(self: String): uptr {
	return self.data.length;
}

func String.at(self: String, idx: uptr): u8 {
	return self.data[idx];
}

func print(string: String) {
	for var i: uptr = 0; i < string.len(); i = i + 1 {
		putchar(string.at(i) as u32);
	}
}