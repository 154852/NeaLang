func exit(code: i32) extern

func main() {
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

	exit(10);
}

func do_something(a: i32): i32 {
	return a + 1;
}