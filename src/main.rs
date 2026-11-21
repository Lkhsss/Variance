use colored::Colorize;
use varstat::Variance;

fn print_help() {
    println!("{}", "varstat".truecolor(0, 191, 255).bold());
    println!("  v{}", env!("CARGO_PKG_VERSION").truecolor(248, 248, 255));
    println!(
        "\t一个简单的计算方差的工具。由 Rust 编写而成。是{}学习 Rust 语言的练手之作。",
        "LKHSSS".truecolor(0, 238, 118)
    );
    println!("\n命令格式：");
    println!("\t.\\varstat [选项] <num1> <num2> ...");
    println!("\n选项：");
    println!("\t-s, --sample      输出无偏样本方差（÷(n-1)）");
    println!("\t-p, --population  输出总体方差（÷n，默认）");
    println!("\t-h, --help        显示本帮助");
    println!("\n示例：");
    println!("\t.\\varstat 1 2 3 4 5 6 7");
    println!("\t.\\varstat --sample 1 2 3 4 5 6 7");
}

fn main() {
    // 从命令行接收数据（跳过程序名本身）
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    // 默认按方差的数学定义（总体方差，÷n）计算
    let mut sample_mode = false;
    let mut numlist: Vec<f64> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "-s" | "--sample" => {
                sample_mode = true;
                continue;
            }
            "-p" | "--population" => {
                sample_mode = false;
                continue;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }

        // 先尝试按数字解析，这样 "-5" 这类负数不会被误判为选项
        match arg.parse::<f64>() {
            Ok(num) if num.is_finite() => numlist.push(num),
            Ok(_) => {
                // 例如 NaN、inf：能解析但不是有效数据
                println!(
                    "{}{}{}{}",
                    "[ERROR]: ".red().bold(),
                    "参数 \"".red(),
                    arg.as_str().red().bold(),
                    "\"不是有限实数!".red()
                );
            }
            Err(_) => {
                println!(
                    "{}{}{}{}",
                    "[ERROR]: ".red().bold(),
                    "参数 \"".red(),
                    arg.as_str().red().bold(),
                    "\"不是有效参数!".red()
                );
            }
        }
    }

    if numlist.is_empty() {
        println!("{} 没有有效参数", "[ERROR]".red().bold());
        return;
    }

    let var = Variance::from_slice(&numlist).compute();
    let len = var.count();

    println!("\n共 {} 项有效数据\n", len.to_string().bright_green());

    // 计算总和
    let sum: f64 = numlist.iter().sum();
    println!("总数：   {}", sum.to_string().truecolor(255, 215, 0));

    println!("平均数： {}", var.mean().to_string().truecolor(0, 238, 118));

    if sample_mode {
        println!(
            "样本方差:{}",
            var.get_sample_variance().to_string().truecolor(0, 191, 255)
        );
        println!(
            "标准差： {} ",
            var.get_sample_standard_deviation()
                .to_string()
                .truecolor(200, 50, 130)
        );
        if len < 2 {
            println!("{} 样本方差至少需要 2 项有效数据", "[WARN]".yellow().bold());
        }
    } else {
        println!(
            "总体方差:{}",
            var.get_variance().to_string().truecolor(0, 191, 255)
        );
        println!(
            "标准差： {} ",
            var.get_standard_deviation()
                .to_string()
                .truecolor(200, 50, 130)
        );
        println!(
            "\n使用 {} 可计算无偏样本方差",
            "--sample".truecolor(0, 238, 118)
        );
    }
}
