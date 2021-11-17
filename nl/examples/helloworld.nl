import std

func main() {
	var other_string = new String;
	var text = new u8[4];
	text[0] = 65;
	text[1] = 66;
	text[2] = 67;
	text[3] = 10;

	other_string.data = text;
	print(other_string);

	print("Hello world\n");
	print("Hello 2\n");
	exit(0);
}