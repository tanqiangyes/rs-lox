use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct TreeType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

pub fn generate_ast(output_dir: &String) -> io::Result<()> {
    define_ast(
        output_dir,
        &"Expr".to_string(),
        &[
            "Binary : Box<Expr> left, Token operator, Box<Expr> right".to_string(),
            "Grouping: Box<Expr> expression".to_string(),
            "Literal : Option<Object> value".to_string(),
            "Unary : Token operator, Box<Expr> right".to_string(),
        ],
    )?;
    Ok(())
}

fn define_ast(output_dir: &String, base_name: &String, types: &[String]) -> io::Result<()> {
    let path = format!("{output_dir}/{}.rs", base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();

    writeln!(file, "use crate::error::*;")?;
    writeln!(file, "use crate::token::*;")?;
    writeln!(file, "use crate::object::Object;")?;

    for ttype in types {
        let (base_class_name, args) = ttype.split_once(':').unwrap();
        let class_name = format!("{}{}", base_class_name.trim(), base_name);

        let args_split = args.split(',');
        let mut fields = Vec::new();
        for arg in args_split {
            let (t2type, name) = arg.trim().split_once(' ').unwrap();

            fields.push(format!("{}: {}", name, t2type));
        }
        tree_types.push(TreeType {
            base_class_name: base_class_name.trim().to_string(),
            class_name,
            fields,
        });
    }

    writeln!(file, "\npub enum {base_name} {{")?;
    for tree_type in &tree_types {
        writeln!(
            file,
            "      {}({}),",
            tree_type.base_class_name, tree_type.class_name
        )?;
    }
    writeln!(file, "}}\n")?;

    writeln!(file, "impl {base_name} {{")?;
    writeln!(
        file,
        "     pub fn accept<R>(&self, visitor: &dyn {}Visitor<R>) -> Result<R, LoxError> {{",
        base_name
    )?;
    writeln!(file, "          match self {{")?;
    for tree_type in &tree_types {
        writeln!(
            file,
            "               {}::{}(be) => be.accept(visitor),",
            base_name, tree_type.base_class_name
        )?;
    }
    writeln!(file, "          }}")?;
    writeln!(file, "     }}")?;
    writeln!(file, "}}\n")?;

    for tree_type in &tree_types {
        writeln!(file, "pub struct {} {{", tree_type.class_name)?;
        for field in &tree_type.fields {
            writeln!(file, "     pub {},", field)?;
        }
        writeln!(file, "}}\n")?;

        writeln!(file, "impl {} {{", tree_type.class_name)?;
        writeln!(
            file,
            "     pub fn accept<R>(&self, visitor: &dyn ExprVisitor<R>) -> Result<R, LoxError> {{"
        )?;
        writeln!(
            file,
            "          visitor.visit_{}_{}(self)",
            tree_type.base_class_name.to_lowercase(),
            base_name.to_lowercase()
        )?;
        writeln!(file, "      }}")?;
        writeln!(file, "}}")?;
    }

    writeln!(file, "pub trait ExprVisitor<R> {{")?;
    for tree_type in &tree_types {
        writeln!(
            file,
            "      fn visit_{}_{}(&self, {}: &{}) -> Result<R, LoxError>; ",
            tree_type.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            base_name.to_lowercase(),
            tree_type.class_name
        )?;
    }
    writeln!(file, "}}")?;

    println!("{:?}", tree_types);

    Ok(())
}
