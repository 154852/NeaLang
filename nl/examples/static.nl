func [location="Std"] putchar(b: i32) extern

func [entry] main() {
	var a: u8 = 1 + 2;
	var c: u8 = a + 8 + other();

	putchar(65);
	putchar(10);
}

func other(): u8 {
	return 5 as u8;
}