func [location="nl/examples/Std"] putchar(b: i32) extern

struct String {
	data: u8[]
}

func [entry] main() {
	var a: u8 = 1 + 2;
	var c: u8 = a + 8 + other();

	var s = new String;
	s.data = new u8[5];

	s.data[0] = 66;
	s.data[1] = 67;
	s.data[2] = 68;
	s.data[3] = 69;
	s.data[4] = 10;

	putchar(s.data[0] as i32);
	putchar(s.data[1] as i32);

	putchar(65);
	putchar(10);

	print(s);
}

func print(string: String) {
	for var i: uptr = 0; i < string.data.length; i = i + 1 {
		putchar(string.data[i] as i32);
	}
}

func other(): u8 {
	return 5 as u8;
}