func [entry] main() {
	var a: u8 = 1 + 2;
	var c: u8 = a + 8 + other();
}

func other(): u8 {
	return 5 as u8;
}