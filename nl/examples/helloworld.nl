func exit(code: i32) extern
func putchar(chr: u32) extern

func main() {
	print("Hello world\n");
	print("Hello 2\n");
    exit(0);
}

func print(string: u8[]) {
	for var i = 0 as uptr; i < string.length; i = i + 1 {
		putchar(string[i] as u32);
	}
}