use std::fmt::Write;

use crate::{Function, Ins, StorableType, ValuePath, ValuePathComponent, ValuePathOrigin, ValueType};

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::U8 => f.write_str("u8"),
            ValueType::I8 => f.write_str("i8"),
            ValueType::U16 => f.write_str("u16"),
            ValueType::I16 => f.write_str("i16"),
            ValueType::U32 => f.write_str("u32"),
            ValueType::I32 => f.write_str("i32"),
            ValueType::U64 => f.write_str("u64"),
            ValueType::I64 => f.write_str("i64"),
            ValueType::UPtr => f.write_str("uptr"),
            ValueType::IPtr => f.write_str("iptr"),
            ValueType::Bool => f.write_str("bool"),
            ValueType::Ref(st) => f.write_fmt(format_args!("ref({})", st)),
            ValueType::Index(st) => f.write_fmt(format_args!("idx({})", st)),
        }
    }
}

impl std::fmt::Display for StorableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorableType::Compound(compound) => f.write_fmt(format_args!("#comp({:?})", compound.name())),
            StorableType::Value(vt) => vt.fmt(f),
            StorableType::Slice(st) => f.write_fmt(format_args!("#slice({})", st)),
            StorableType::SliceData(st) => f.write_fmt(format_args!("#slicedata({})", st)),
        }
    }
}

impl std::fmt::Display for ValuePathOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValuePathOrigin::Local(local_index, _) => f.write_fmt(format_args!("#lcl({})", local_index)),
            ValuePathOrigin::Global(global_index, _) => f.write_fmt(format_args!("#glbl({})", global_index)),
            ValuePathOrigin::Deref(_) => f.write_str("deref"),
        }
    }
}

impl std::fmt::Display for ValuePathComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValuePathComponent::Slice(_) => f.write_str("slice"),
            ValuePathComponent::Property(idx, _, _) => f.write_fmt(format_args!("prop({})", idx)),
            ValuePathComponent::Length => f.write_str("length"),
        }
    }
}

impl std::fmt::Display for ValuePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.origin().fmt(f)?;
        for component in self.components() {
            f.write_char('/')?;
            component.fmt(f)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Ins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ins::PushPath(path, vt) => f.write_fmt(format_args!("pushpath %{} {}", vt, path)),
            Ins::Push(vt) => f.write_fmt(format_args!("push %{}", vt)),
            Ins::Pop(vt) => f.write_fmt(format_args!("pop %{}", vt)),
            Ins::Index(vt) => f.write_fmt(format_args!("index %{}", vt)),
            Ins::New(st) => f.write_fmt(format_args!("new {}", st)),
            Ins::NewSlice(st) => f.write_fmt(format_args!("newslice {}", st)),
            Ins::Convert(from, to) => f.write_fmt(format_args!("conv {}, {}", from, to)),
            Ins::Call(idx) => f.write_fmt(format_args!("call #fn({})", idx)),
            Ins::Ret => f.write_str("ret"),
            Ins::Inc(vt, i) => f.write_fmt(format_args!("inc %{} {}", vt, i)),
            Ins::Dec(vt, i) => f.write_fmt(format_args!("dec %{} {}", vt, i)),
            Ins::Add(vt) => f.write_fmt(format_args!("add %{}", vt)),
            Ins::Mul(vt) => f.write_fmt(format_args!("mul %{}", vt)),
            Ins::Div(vt) => f.write_fmt(format_args!("div %{}", vt)),
            Ins::Sub(vt) => f.write_fmt(format_args!("sub %{}", vt)),
            Ins::Eq(vt) => f.write_fmt(format_args!("eq %{}", vt)),
            Ins::Ne(vt) => f.write_fmt(format_args!("ne %{}", vt)),
            Ins::Lt(vt) => f.write_fmt(format_args!("lt %{}", vt)),
            Ins::Le(vt) => f.write_fmt(format_args!("le %{}", vt)),
            Ins::Gt(vt) => f.write_fmt(format_args!("gt %{}", vt)),
            Ins::Ge(vt) => f.write_fmt(format_args!("ge %{}", vt)),
            Ins::Loop(code, cond, inc) => {
                f.write_str("loop\n\tcode {")?;
                for ins in code {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}\n\tcond {")?;
                for ins in cond {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}\n\tinc {")?;
                for ins in inc {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}")?;
                Ok(())
            },
            Ins::If(true_then, cond) => {
                f.write_str("if\n\tthen {")?;
                for ins in true_then {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}\n\tcond {")?;
                for ins in cond {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}")?;
                Ok(())
            },
            Ins::IfElse(true_then, false_then, cond) => {
                f.write_str("if\n\tthen {")?;
                for ins in true_then {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}\n\telse {")?;
                for ins in false_then {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}\n\tcond {")?;
                for ins in cond {
                    f.write_str("\n\t\t")?;
                    f.write_str(&format!("{}", ins).replace('\n', "\n\t\t"))?;
                }
                f.write_str("\n\t}")?;
                Ok(())
            },
            Ins::Break(depth) => f.write_fmt(format_args!("break {}", depth)),
            Ins::Continue(depth) => f.write_fmt(format_args!("break {}", depth)),
            Ins::PushLiteral(vt, val) => f.write_fmt(format_args!("pushlit %{} {}", vt, val)),
            Ins::Drop => f.write_str("drop"),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code_opt() {
            f.write_char('{')?;
            for ins in code {
                f.write_str("\n\t")?;
                f.write_str(&format!("{}", ins).replace('\n', "\n\t"))?;
            }
            f.write_str("\n}")?;
        } else {
            f.write_str("extern")?;
        }

        Ok(())
    }
}