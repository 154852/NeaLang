func exit(code: i32) extern
func putchar(chr: u32) extern
func nl_new_object(size: uptr): uptr extern
func nl_new_slice(count: uptr, size: uptr): uptr extern

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
	var other_string = new String;
	var text = new u8[4];
	other_string.data = text;
	text[0] = 65;
	text[1] = 66;
	text[2] = 67;
	text[3] = 10;

	print(other_string);

	print("Hello world\n");
	print("Hello 2\n");
    exit(0);
}

func print(string: String) {
	for var i = 0 as uptr; i < string.len(); i = i + 1 {
		putchar(string.at(i) as u32);
	}
}