import std

func main() {
	var other_string = new String;
	var text = new u8[4 as uptr];
	text[0 as uptr] = 65 as u8;
	text[1 as uptr] = 66 as u8;
	text[2 as uptr] = 67 as u8;
	text[3 as uptr] = 10 as u8;

	other_string.data = text;
	print(other_string);

	print("Hello world\n");
	print("Hello 2\n");
    exit(0 as i32);
}