use colored::Colorize;

fn main() {
    // 从命令行接收数据
    let args: Vec<String> = std::env::args().collect();
    let mut numlist = Vec::new();

    // 判断是否有参数
    if args.len() < 2 {
        println!(
            "{}{}{}{}{}",
            "Variance".truecolor(0, 191, 255).bold(),
            "\n    一个简单的计算方差的工具。由Rust编写而成。是",
            "LKHSSS".truecolor(0, 238, 118),
            "学习Rust语言的练手之作。",
            "\n命令格式： \n    .\\variance 1 2 3 4 5 6 7"
        );
    } else {
        for arg in args.iter().skip(1) {
            let num: Result<f64, _> = arg.parse();
            match num {
                Ok(num) => {
                    // 如果 num 是 Ok 类型，说明 arg 是一个数字，可以继续执行后续操作
                    numlist.push(num);
                }
                Err(_) => {
                    // 如果 num 是 Err 类型，说明 arg 不是一个数字，需要报错
                    println!(
                        "{}{}{}{}",
                        "[ERROR]: ".red().bold(),
                        "参数 \"".red(),
                        arg.to_string().red().bold(),
                        "\"不是有效参数!".red()
                    );
                }
            };
        }

        // 打印接收到的数据
        // println!("接收到的数据：{:?}", numlist);

        variance(numlist);
    }
}

fn variance(numlist: Vec<f64>) -> f64 {
    // dbg!(&numlist);
    //计算有多少项
    let len = numlist.len() as u64;
    // \033[字背景;前景m
    // \033[38;2;<r>;<g>;<b>m
    println!("\n共 {} 项有效数据\n", &len.to_string().red());

    // 计算总和
    let sum: f64 = numlist.iter().sum();
    println!("总数：   {}", &sum.to_string().truecolor(255, 215, 0));

    //计算平均数
    let avg = sum as f64 / (len as f64);
    println!("平均数： {}", &avg.to_string().truecolor(0, 238, 118));

    //计算所有元素与其平均值的平方差的总和
    let allsum: f64 = numlist.iter().map(|x| (x - avg) * (x - avg)).sum();

    // 计算方差
    let variance = allsum / len as f64;

    println!("方差：   {}", &variance.to_string().truecolor(0, 191, 255));

    variance
}
