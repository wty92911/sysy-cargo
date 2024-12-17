use compiler::parser::asm::visitor::{*, Visitor};
use koopa::back::{KoopaGenerator, NameManager, Visitor as BackVisitor};
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{read_to_string, File};
use std::io::{Result, Write};

// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy

lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let (mode, input, output) = parse_args();
    // 读取输入文件
    let input = read_to_string(input)?;
    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&dbg!(input)).unwrap();
    let program = ast.into();
    let mut file = File::create(output)?;
    let mut text = "Not implemented".to_string();
    match mode.as_str() {
        "-koopa" => {
            // convert to text form
            let mut gen = KoopaGenerator::new(Vec::new());
            gen.generate_on(&program)?;
            text = std::str::from_utf8(&gen.writer()).unwrap().to_string();
            // println!("{}", text_form_ir);
            
        }
        "-riscv" => {
            let mut asm_visitor = Visitor::default();
            let mut name_manager = NameManager::default();
            let mut riscv_code = Vec::new();
            asm_visitor.visit(&mut riscv_code, &mut name_manager, &program)?;
            text = String::from_utf8(riscv_code).unwrap();
        }
        _ => {panic!("Not implement")}
    }
    write!(file, "{}", text)?;

    Ok(())
}

fn parse_args() -> (String, String, String) {
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();
    (mode, input, output)
}