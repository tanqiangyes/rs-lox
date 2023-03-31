use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct TreeType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

pub fn generate_ast(output_dir: &str) -> io::Result<()> {
    define_ast(
        output_dir,
        "Expr",
        &["error", "token", "object"],
        &[
            "Assign: Token name, Box<Expr> value",
            "Binary : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping: Box<Expr> expression",
            "Literal : Option<Object> value",
            "Logical: Box<Expr> left, Token operator, Box<Expr> right",
            "Unary : Token operator, Box<Expr> right",
            "Variable: Token name",
        ],
    )?;
    define_ast(
        output_dir,
        "Stmt",
        &["error", "expr", "token"],
        &[
            "Block: Vec<Stmt> statements",
            "Expression: Expr expression",
            "If: Expr condition, Box<Stmt> then_branch, Option<Box<Stmt>> else_branch",
            "Print : Expr expression",
            "Var : Token name, Option<Expr> initializer",
            "While: Expr condition, Box<Stmt> body",
        ],
    )?;
    Ok(())
}

fn define_ast(
    output_dir: &str,
    base_name: &str,
    imports: &[&str],
    types: &[&str],
) -> io::Result<()> {
    let path = format!("{output_dir}/{}.rs", base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();

    for i in imports {
        writeln!(file, "use crate::{}::*;", i)?;
    }

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

    writeln!(file, "\n#[derive(Debug)]")?;
    writeln!(file, "pub enum {base_name} {{")?;
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
        writeln!(file, "\n#[derive(Debug)]")?;
        writeln!(file, "pub struct {} {{", tree_type.class_name)?;
        for field in &tree_type.fields {
            writeln!(file, "     pub {},", field)?;
        }
        writeln!(file, "}}\n")?;

        writeln!(file, "impl {} {{", tree_type.class_name)?;
        writeln!(
            file,
            "     pub fn accept<R>(&self, visitor: &dyn {}Visitor<R>) -> Result<R, LoxError> {{",
            base_name
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

    writeln!(file, "pub trait {}Visitor<R> {{", base_name)?;
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
