import std

func test_new() {
	var string = new String;
	var data = new u8[10];
	string.data = data;

	if string.len() == 10 {
		test_pass("test_new");
	} else {
		test_fail("test_new");
	}
}

func test_int_slice_index() {
	var u8_data = new u8[10];
	u8_data[0] = 5;
	u8_data[1] = 6;
	var x: u8 = u8_data[5];

	var i32_data = new i32[4];
	i32_data[3] = 9;
	var x: i32 = i32_data[1];

	if u8_data[0] == 5 {
		if i32_data[3] == 9 {
			test_pass("test_int_slice_index");
			return;
		}
	}
	
	test_fail("test_int_slice_index");
}

func test_if() {
	if 1 < 2 {
		if 2 > 1 {
			test_pass("test_if");
			return;
		}
	}
	test_fail("test_if");
}

func test_if_else() {
	if 1 < 2 {
		if 2 < 1 {

		} else {
			test_pass("test_if_else");
			return;
		}
	} else {

	}
	test_fail("test_if_else");
}

func test_bool_expr() {
	if 1 + 2 == 3 && 1 == 1 {
		if 1 + 2 == 1 || 1 == 1 {
			if 1 + 2 == 1 || 2 == 1 {

			} else {
				test_pass("test_bool_expr");
				return;
			}
		}
	}
	test_fail("test_bool_expr");
}

func test_math() {
	if 1 + 2 == 3 {
		if 3 * 4 == 12 {
			if 6 / 3 == 2 {
				if 9 - 8 == 1 {
					test_pass("test_math");
					return;
				}
			}
		}
	}

	test_fail("test_math");
}

func test_div() {
	if (4 / 2) == 2 {
		if (9 + 3) / 4 == 3 {
			if 3 == 1 + (4 / (1 + 1)) {
				test_pass("test_div");
				return;
			}
		}
	}

	test_fail("test_div");
}

func a(): i32 {
	return 1 + 2;
}

func b(): i32 {
	return 1 - 2;
}

func test_call_expr() {
	if (a() + b()) == 2 {
		test_pass("test_call_expr");
	} else {
		test_fail("test_call_expr");
	}
}

func test_for() {
	var test = 0;

	for var i = 1; i <= 100; i = i + 1 {
		test = test + i;
	}

	var expected = 100 * (100 + 1) / 2;
	if test == expected {
		test_pass("test_for");
	} else {
		test_fail("test_for");
	}
}

func test_op_order() {
	if 3*2 + 1 == 7 {
		if 9/3 + 2 == 5 {
			if 9/3 + 6*2 == 15 {
				test_pass("test_op_order");
				return;
			}
		}
	}

	test_fail("test_op_order");
}

func test_static() {
	var x = String.empty();

	if x.len() == 0 {
		test_pass("test_static");
	} else {
		test_fail("test_static");
	}
}

func test_slice_literal() {
	var x = [1, 2, 3, 4];

	if x.length == 4 && x[0] == 1 && x[2] == 3 {
		test_pass("test_slice_literal");
	} else {
		test_fail("test_slice_literal");
	}
}

func test_binary_search() {
	if binary_search([1, 2, 3, 4, 5, 6, 7], 4) == 3 && binary_search([1, 2, 3, 4, 5, 6, 7], 1) == 0 {
		test_pass("test_binary_search");
	} else {
		test_fail("test_binary_search");
	}
}

func test_bubble_sort() {
	var a = [6, 4, 3, 2, 1, 5];
	var b = [1, 2, 3, 4, 5, 6];
	
	bubble_sort(a);

	var is_ok = true;
	for var i: uptr = 0; i < a.length; i = i + 1 {
		if a[i] != b[i] {
			is_ok = false;
		}
	}

	if is_ok {
		test_pass("test_bubble_sort");
	} else {
		test_fail("test_bubble_sort");
	}
}

func test_drop() {
	var x = new String;
	drop x;

	var y = new u8[7];
	drop y;

	test_pass("test_drop");
}

func [entry] main() {
	test_if();
	test_if_else();
	test_bool_expr();
	test_new();
	test_int_slice_index();
	test_math();
	test_call_expr();
	test_for();
	test_div();
	test_op_order();
	test_static();
	test_slice_literal();
	test_binary_search();
	test_bubble_sort();
	test_drop();
}