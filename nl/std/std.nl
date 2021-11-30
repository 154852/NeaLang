func [arch="x86"] exit(code: i32) extern
func [arch="x86"] putchar(chr: u32) extern
func [arch="x86", alloc] nl_new_object(size: uptr): uptr extern
func [arch="x86", alloc_slice] nl_new_slice(count: uptr, size: uptr): uptr extern

func [arch="wasm", location="core"] exit(code: i32) extern
func [arch="wasm", location="core"] putchar(chr: u32) extern
func [arch="wasm", location="core", alloc] new_object(size: uptr): uptr extern
func [arch="wasm", location="core", alloc_slice] new_slice(count: uptr, size: uptr): uptr extern

func [arch="java", location="nl/std/Std"] exit(code: i32) extern
func [arch="java", location="nl/std/Std"] putchar(b: u32) extern

struct String {
	data: u8[]
}

func String.empty(): String {
	var string: String = new String;
	string.data = new u8[0];

	return string;
}

func String.len(self): uptr {
	return self.data.length;
}

func String.at(self, idx: uptr): u8 {
	return self.data[idx];
}

func print(string: String) {
	for var i: uptr = 0; i < string.len(); i = i + 1 {
		putchar(string.at(i) as u32);
	}
}

func colour_green() {
	putchar(27);
	print("[32m");
}

func colour_red() {
	putchar(27);
	print("[31m");
}

func colour_clear() {
	putchar(27);
	print("[0m");
}

func test_pass(name: String) {
	print(name);
	print(" - ");
	colour_green();
	print("passed");
	colour_clear();
	print("\n");
}

func test_fail(name: String) {
	print(name);
	print(" - ");
	colour_red();
	print("failed");
	colour_clear();
	print("\n");
}