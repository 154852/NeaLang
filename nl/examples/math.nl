func main() {
	var a: i32 = 1 + 2;
	var b = do_something(10);

	do_something(11);

	if 5 {
		a = a + 1;
	} else {
		a = a - 1;
	}

	b = b + 1;
}

func do_something(a: i32): i32 {
	return a + 1;
}