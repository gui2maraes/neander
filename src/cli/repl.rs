use std::fs;
use std::{path::Path, process::ExitCode};

use crate::cpu::{ExecResult, Neander};
use crate::memfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directive {
    Step,
    StepN(u32),
    BreakPoint(u8),
    Clear(u8),
    Continue,
    PrintCpu,
    PrintMemAddr(u8),
    PrintMemRange(u8, u8),
    Help,
    Quit,
}

pub fn run_repl(file: &Path) -> ExitCode {
    let mut cpu = Neander::new();
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = memfile::parse_memfile(cpu.memory_mut(), &source) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    let mut buf = String::new();
    let mut bps = [false; 256];
    let mut last_dir = None;
    loop {
        // read directive
        buf.clear();
        let dir = match std::io::stdin().read_line(&mut buf).unwrap() {
            // if 1 byte was read, it was a newline. Repeat last directive
            1 => match last_dir {
                Some(d) => d,
                None => continue,
            },
            // if EOF, quit session
            0 => break,
            // else, parse the line
            _ => match parser::parse_directive(&buf) {
                Ok(d) => d,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            },
        };
        last_dir = Some(dir);
        match dir {
            Directive::Quit => break,
            Directive::Help => print_help(),
            Directive::BreakPoint(x) => {
                if bps[x as usize] {
                    println!("breakpoint already set at {x}");
                } else {
                    bps[x as usize] = true;
                    println!("breakpoint set at {x}");
                }
            }
            Directive::Clear(x) => {
                if !bps[x as usize] {
                    println!("no breakpoint at {x}");
                } else {
                    bps[x as usize] = false;
                    println!("cleared breakpoint at {x}");
                }
            }
            Directive::PrintCpu => {
                println!("{cpu}");
            }
            Directive::PrintMemAddr(a) => {
                println!("{0} | {0:X} | {0:b}", cpu.memory()[a as usize]);
            }
            Directive::PrintMemRange(a, b) => {
                cpu.print_mem_range(a, b);
            }
            Directive::Step => match cpu.step() {
                ExecResult::Halted => println!("end of program reached"),
                ExecResult::Normal => println!("{cpu}"),
                ExecResult::MemWrite { addr, value } => println!("{cpu}\nmem[{addr}] <- {value}"),
                ExecResult::Exception(e) => {
                    println!("exception: {e}");
                    break;
                }
            },
            Directive::StepN(n) => {
                for _ in 0..n {
                    match cpu.step() {
                        ExecResult::Halted => {
                            println!("end of program reached");
                            break;
                        }
                        ExecResult::MemWrite { addr, value } => {
                            println!("mem[{addr}] <- {value}");
                            if bps[cpu.pc() as usize] {
                                println!("breakpoint reached");
                                break;
                            }
                        }
                        ExecResult::Normal => {
                            if bps[cpu.pc() as usize] {
                                println!("breakpoint reached");
                                break;
                            }
                        }
                        ExecResult::Exception(e) => {
                            println!("exception: {e}");
                            break;
                        }
                    }
                }
            }
            Directive::Continue => loop {
                match cpu.step() {
                    ExecResult::Halted => {
                        println!("end of program reached");
                        break;
                    }
                    ExecResult::MemWrite { addr, value } => {
                        println!("mem[{addr}] <- {value}");
                        if bps[cpu.pc() as usize] {
                            println!("breakpoint reached");
                            break;
                        }
                    }
                    ExecResult::Normal => {
                        if bps[cpu.pc() as usize] {
                            println!("breakpoint reached");
                            break;
                        }
                    }
                    ExecResult::Exception(e) => {
                        println!("exception: {e}");
                        break;
                    }
                }
            },
        }
    }
    ExitCode::SUCCESS
}
fn print_help() {
    println!(
        "valid directives:
         - help, h: display this help
         - step, s: execute the next instruction
         - (step, s) n: execute the next n instructions
         - (breakpoint, b) i: set a breakpoint at instruction i
         - (clear, cl) i: clear a breakpoint at instruction i
         - continue, c: continue execution until next breakpoint
         - cpu, show, print: print CPU content
         - mem: print all memory
         - mem (addr, start.., ..end, start..end): print memory in address or supplied range
         - quit, q: quit session"
    )
}

mod parser {
    use std::str::FromStr;

    use super::Directive;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::bytes::complete::take_while1;
    use nom::character::complete::digit1;
    use nom::combinator::eof;
    use nom::combinator::map_res;
    use nom::combinator::opt;
    use nom::sequence::{pair, preceded};
    use nom::{IResult, Parser};

    pub fn parse_directive(input: &str) -> Result<Directive, &'static str> {
        match directive(input) {
            Ok((rest, dir)) if rest.is_empty() => Ok(dir),
            _ => Err("Invalid directive. For valid directives, type `help`"),
        }
    }

    fn directive(input: &str) -> IResult<&str, Directive> {
        alt((quit, cont, step, mem, cpu, breakpoint, clear, help)).parse(input.trim())
    }

    fn help(input: &str) -> IResult<&str, Directive> {
        word("h")
            .or(word("help"))
            .map(|_| Directive::Help)
            .parse(input)
    }

    fn mem(input: &str) -> IResult<&str, Directive> {
        let end_range = preceded(tag(".."), uint::<u8>).map(|x| Directive::PrintMemRange(0, x));
        let range =
            pair(uint::<u8>, opt(preceded(tag(".."), opt(uint::<u8>)))).map(|(a, n)| match n {
                None => Directive::PrintMemAddr(a),
                Some(None) => Directive::PrintMemRange(a, 255),
                Some(Some(b)) => Directive::PrintMemRange(a, b),
            });

        let range = end_range.or(range);
        let mem = word("mem").map(|_| Directive::PrintMemRange(0, 255));
        let mem_range = pair(word("mem"), range).map(|(_, d)| d);
        mem_range.or(mem).parse(input)
    }
    fn quit(input: &str) -> IResult<&str, Directive> {
        word("quit")
            .or(word("q"))
            .map(|_| Directive::Quit)
            .parse(input)
    }
    fn cpu(input: &str) -> IResult<&str, Directive> {
        word("cpu").map(|_| Directive::PrintCpu).parse(input)
    }
    fn cont(input: &str) -> IResult<&str, Directive> {
        word("continue")
            .or(word("c"))
            .map(|_| Directive::Continue)
            .parse(input)
    }
    //fn parse_directive(input: &str) -> Result<Directive, &str> {}
    fn breakpoint(input: &str) -> IResult<&str, Directive> {
        let bp = word("breakpoint").or(word("b")).or(word("bp"));
        let pc = uint::<u8>;
        pair(bp, pc)
            .map(|(_, x)| Directive::BreakPoint(x))
            .parse(input)
    }
    fn clear(input: &str) -> IResult<&str, Directive> {
        let bp = word("clear").or(word("cl"));
        let pc = uint::<u8>;
        pair(bp, pc).map(|(_, x)| Directive::Clear(x)).parse(input)
    }
    fn step(input: &str) -> IResult<&str, Directive> {
        let step_n = pair(word("step").or(word("s")), uint).map(|(_, n)| Directive::StepN(n));
        let step = word("step").or(word("s")).map(|_| Directive::Step);
        alt((step_n, step))(input)
    }
    fn uint<T: FromStr>(input: &str) -> IResult<&str, T> {
        map_res(digit1, str::parse)(input)
    }
    fn space(input: &str) -> IResult<&str, ()> {
        take_while1(|c: char| c.is_whitespace())
            .map(|_| ())
            .parse(input)
    }
    fn word(word: &str) -> impl Parser<&str, (), nom::error::Error<&str>> {
        use nom::sequence::terminated;
        terminated(tag(word), space.or(eof.map(|_| ()))).map(|_| ())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn parse_step() {
            assert_eq!(step("step 10"), Ok(("", Directive::StepN(10))));
            assert_eq!(step("step"), Ok(("", Directive::Step)));
        }
        #[test]
        fn parse_breakpoint() {
            assert_eq!(
                breakpoint("breakpoint 10"),
                Ok(("", Directive::BreakPoint(10)))
            );
            assert!(breakpoint("breakpoint -1").is_err());
            assert!(breakpoint("breakpoint").is_err());
        }
        #[test]
        fn parse_mem() {
            assert_eq!(mem("mem"), Ok(("", Directive::PrintMemRange(0, 255))));
            assert_eq!(mem("mem 10.."), Ok(("", Directive::PrintMemRange(10, 255))));
            assert_eq!(mem("mem ..100"), Ok(("", Directive::PrintMemRange(0, 100))));
            assert_eq!(mem("mem 10"), Ok(("", Directive::PrintMemAddr(10))));
        }

        #[test]
        fn parse_word() {
            assert_eq!(word("abc").parse("abc"), Ok(("", ())));
            assert_eq!(word("abcd").parse("abcd  1"), Ok(("1", ())));
            assert!(word("abc").parse("abcdef").is_err());
        }

        #[test]
        fn test_directive() {
            assert_eq!(parse_directive("c"), Ok(Directive::Continue));
            assert_eq!(parse_directive("continue"), Ok(Directive::Continue));
            assert_eq!(parse_directive("h"), Ok(Directive::Help));
            assert_eq!(parse_directive("help"), Ok(Directive::Help));
            assert_eq!(parse_directive("cpu"), Ok(Directive::PrintCpu));
            assert_eq!(parse_directive("mem"), Ok(Directive::PrintMemRange(0, 255)));
            assert!(parse_directive("c a").is_err());
            assert!(parse_directive("continue 1").is_err());
        }
    }
}
