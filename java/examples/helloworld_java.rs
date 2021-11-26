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

	let system_out = {
		let system_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("java/lang/System")));
		let system = classfile.add_constant(java::Constant::Class(java::Class::new(system_utf8)));
		
		let out_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("out")));
		let out_desc_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("Ljava/io/PrintStream;")));
		let out = classfile.add_constant(java::Constant::NameAndType(java::NameAndType::new(
			out_utf8, out_desc_utf8
		)));
		
		classfile.add_constant(java::Constant::FieldRef(java::FieldRef::new(
			system, out
		)))
	};

	let printstream_println = {
		let printstream_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("java/io/PrintStream")));
		let printstream = classfile.add_constant(java::Constant::Class(java::Class::new(printstream_utf8)));
		
		let println_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("println")));
		let println_desc_utf8 = classfile.add_constant(java::Constant::Utf8(java::Utf8::new("(Ljava/lang/String;)V")));
		let println = classfile.add_constant(java::Constant::NameAndType(java::NameAndType::new(
			println_utf8, println_desc_utf8
		)));
		
		classfile.add_constant(java::Constant::MethodRef(java::MethodRef::new(
			printstream, println
		)))
	};

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