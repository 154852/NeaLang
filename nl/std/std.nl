func [arch="linux-x86"] exit(code: i32) extern
func [arch="linux-x86"] putchar(chr: u32) extern
func [arch="linux-x86", alloc] nl_new_object(size: uptr): uptr extern
func [arch="linux-x86", alloc_slice] nl_new_slice(count: uptr, size: uptr): uptr extern
func [arch="linux-x86", free] nl_drop_object(object: uptr, size: uptr) extern
func [arch="linux-x86", free_slice] nl_drop_slice(slice: uptr, element_size: uptr) extern

func [arch="macos-x86"] exit(code: i32) extern
func [arch="macos-x86"] putchar(chr: u32) extern
func [arch="macos-x86", alloc] nl_new_object(size: uptr): uptr extern
func [arch="macos-x86", alloc_slice] nl_new_slice(count: uptr, size: uptr): uptr extern
func [arch="macos-x86", free] nl_drop_object(object: uptr, size: uptr) extern
func [arch="macos-x86", free_slice] nl_drop_slice(slice: uptr, element_size: uptr) extern

func [arch="macos-arm64"] exit(code: i32) extern
func [arch="macos-arm64"] putchar(chr: u32) extern
func [arch="macos-arm64", alloc] nl_new_object(size: uptr): uptr extern
func [arch="macos-arm64", alloc_slice] nl_new_slice(count: uptr, size: uptr): uptr extern
func [arch="macos-arm64", free] nl_drop_object(object: uptr, size: uptr) extern
func [arch="macos-arm64", free_slice] nl_drop_slice(slice: uptr, element_size: uptr) extern

func [arch="wasm", location="core"] exit(code: i32) extern
func [arch="wasm", location="core"] putchar(chr: u32) extern
func [arch="wasm", location="core", alloc] new_object(size: uptr): uptr extern
func [arch="wasm", location="core", alloc_slice] new_slice(count: uptr, size: uptr): uptr extern
func [arch="wasm", location="core", free] drop_object(object: uptr, size: uptr) extern
func [arch="wasm", location="core", free_slice] drop_slice(slice: uptr, element_size: uptr) extern

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

func println(string: String) {
	print(string);
	putchar(10);
}

func printi(i: i32) {
	if i == 0 {
		print("0");
		return;
	}

	if i < 0 {
		print("-");
		i = -i;
	}

	var pow = 1;
	for pow <= i {
		pow = pow * 10;
	}

	pow = pow / 10;

	for pow > 0 {
		putchar(48 + (i / pow) as u32);
		i = i - ((i / pow) * pow);
		pow = pow / 10;
	}
}

func printiln(i: i32) {
	printi(i);
	putchar(10);
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

func binary_search(sorted_slice: i32[], target: i32): uptr {
	var start = 0 as uptr;
	var end = sorted_slice.length - 1;

	for {
		var middle = (start + end) / 2;
		var element = sorted_slice[middle];
		if element == target {
			return middle;
		} else if element > target {
			end = middle - 1;
		} else {
			start = middle + 1;
		}

		if end < start { return sorted_slice.length; }
	}

	return 0 as uptr;
}

func bubble_sort(slice: i32[]) {
	var n = slice.length;
	
	for n > 1 {
		var new_n: uptr = 0;
		for var i: uptr = 1; i <= n - 1; i = i + 1 {
			if slice[i - 1] > slice[i] {
				var tmp = slice[i - 1];
				slice[i - 1] = slice[i];
				slice[i] = tmp;

				new_n = i;
			}
		}

		n = new_n;
	}
}