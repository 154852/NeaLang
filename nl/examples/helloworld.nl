func exit(code: i32) extern
func putchar(chr: u32) extern

struct String {
	data: u8[]
}

func String.len(self: String): uptr {
	return self.data.length;
}

func String.at(self: String, idx: uptr): u8 {
	return self.data[idx];
}

func main() {
	print("Hello world\n");
	print("Hello 2\n");
    exit(0);
}

func print(string: String) {
	for var i = 0 as uptr; i < string.len(); i = i + 1 {
		putchar(string.at(i) as u32);
	}
}