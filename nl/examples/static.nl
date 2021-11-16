struct String {
	data: u8[]
}

func String.len(self: String): uptr {
	return self.data.length;
}

func String.at(self: String, idx: uptr): u8 {
	return self.data[idx];
}

func putchar(chr: u8) extern

func main() {
	var a = 1 + 2;
	var c = a + 8;

	var x = "abc";
	print(x);
}

func print(string: String) {
	for var i: uptr = 0; i < string.data.length; i = i + 1 {
		putchar(string.data[i]);
	}
}