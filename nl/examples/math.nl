func exit(code: i32) extern
func putchar(chr: u32) extern
func nl_alloc_slice_u8(size: uptr): u8[] extern
func nl_alloc_slice_u32(size: uptr): u32[] extern

struct String {
	length: i32
}

func main() {
	print("Hello world\n");

	var string = nl_alloc_slice_u8(3);

	string[0] = 65;
	string[1] = 66;
	string[2] = 10;

	print(string);

	var a: i32 = 1 + 2;
	var b = do_something(10);

	do_something(11);

	if 1 < 2 {
		a = a + 1;
	} else {
		a = a - 1;
	}

	for var i = 0; i < 10; i = i + 1 {
		a = a + 1;
	}

	b = b + 1;

	exit(a);
}

func do_something(a: i32): i32 {
	return a + 1;
}

func test_structs() {
	var string: String;

	string.length = 4;
	var x = string.length;
}

func test_slices() {
	var a: u8[];

	var len = a.length;
	var x: u64 = 10;

	var y = a[5];
}

func print(string: u8[]) {
	for var i = 0 as uptr; i < string.length; i = i + 1 {
		putchar(string[i] as u32);
	}
}