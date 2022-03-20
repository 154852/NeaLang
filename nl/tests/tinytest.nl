func putchar(chr: i32) extern

struct String {
	data: u8[]
}

func [entry] main() {
	var a = "Hello world!";

	putchar(a.data[0] as i32);
}