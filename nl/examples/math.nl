func main() {
	var a: i32 = 1 + 2;
	var b = do_something(10);

	do_something(11);

	b = b + 1;
}

func do_something(a: i32): i32 {
	return a + 1;
}