func exit(code: i32) extern
func putchar(chr: u32) extern

struct String {
	data: u8[]
}

func string_len(self: String): uptr {
	return self.data.length;
}

func main() {
	print("Hello world\n");
	print("Hello 2\n");
    exit(0);
}

func print(string: String) {
	for var i = 0 as uptr; i < string.data.length; i = i + 1 {
		putchar(string.data[i] as u32);
	}
}