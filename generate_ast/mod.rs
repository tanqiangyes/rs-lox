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
        &vec![
            "Binary : Box<Expr> left, Token operator, Box<Expr> right".to_string(),
            "Grouping: Box<Expr> expression".to_string(),
            "Literal : Object value".to_string(),
            "Unary : Token operator, Box<Expr> right".to_string(),
        ],
    )?;
    Ok(())
}

fn define_ast(output_dir: &String, base_name: &String, types: &[String]) -> io::Result<()> {
    let path = format!("{output_dir}/{}.rs", base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();

    write!(file, "{}", "use crate::error::*;\n")?;
    write!(file, "{}", "use crate::token::*;\n")?;

    for ttype in types {
        let (base_class_name, args) = ttype.split_once(":").unwrap();
        let class_name = format!("{}{}", base_class_name.trim(), base_name);

        let args_split = args.split(",");
        let mut fields = Vec::new();
        for arg in args_split {
            let (t2type, name) = arg.trim().split_once(" ").unwrap();

            fields.push(format!("{}: {}", name, t2type));
        }
        tree_types.push(TreeType {
            base_class_name: base_class_name.trim().to_string(),
            class_name,
            fields,
        });
    }

    write!(file, "\npub enum {base_name} {{\n")?;
    for tree_type in &tree_types {
        write!(
            file,
            "      {}({}),\n",
            tree_type.base_class_name, tree_type.class_name
        )?;
    }
    write!(file, "}}\n\n")?;

    for tree_type in &tree_types {
        write!(file, "pub struct {} {{\n", tree_type.class_name)?;
        for field in &tree_type.fields {
            write!(file, "      {},\n", field)?;
        }
        write!(file, "}}\n\n")?;

        write!(file, "impl {} {{\n", tree_type.class_name)?;
        write!(
            file,
            "      fn accept<R>(&self, visitor: &dyn ExprVisitor<R>) -> Result<R, LoxError> {{\n"
        )?;
        write!(
            file,
            "          visitor.visit_{}_{}(self);\n",
            tree_type.base_class_name.to_lowercase(),
            base_name.to_lowercase()
        )?;
        write!(file, "      }}\n")?;
        write!(file, "}}\n")?;
    }

    write!(file, "pub trait ExprVisitor<R> {{\n")?;
    for tree_type in &tree_types {
        write!(
            file,
            "      fn visit_{}_{}(&self, {}: &{}) -> Result<R, LoxError>; \n",
            tree_type.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            base_name.to_lowercase(),
            tree_type.class_name
        )?;
    }
    write!(file, "}}")?;

    println!("{:?}", tree_types);

    Ok(())
}
