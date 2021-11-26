use java::Ins;

fn main() {
	let mut classfile = java::ClassFile::new("HelloWorld");

	let string = {
		let utf8_index = classfile.add_constant(java::Constant::Utf8(
			java::Utf8::new("Hello World!")
		));

		classfile.add_constant(java::Constant::String(
			java::JavaString::new(utf8_index)
		))
	};

	let system_out = classfile.const_field("java/lang/System", "out", "Ljava/io/PrintStream;");
	let printstream_println = classfile.const_method("java/io/PrintStream", "println", "(Ljava/lang/String;)V");

	let main = java::Method::new_on("main", "([Ljava/lang/String;)V", &mut classfile);
	main.set_access(java::MethodAccessFlags::from_bits(
		java::MethodAccessFlags::ACC_PUBLIC | java::MethodAccessFlags::ACC_STATIC
	));

	main.add_code(java::Code::new(
		2, 1,
		vec![
			Ins::GetStatic { index: system_out },
			Ins::Ldc { index: string },
			Ins::InvokeVirtual { index: printstream_println },
			Ins::Return
		]
	));

	// Run with `java -classpath java/examples HelloWorld`
	// Or dump with `javap -v java/examples/HelloWorld.class`
	std::fs::write("java/examples/HelloWorld.class", classfile.encode()).expect("Could not write");
}