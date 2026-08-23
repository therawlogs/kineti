//! Validation layer (10XE C3 / ASI-02, D1): every tool result is checked for
//! structural sanity before it re-enters reasoning. Fail → error-context
//! observation → repeated failure escalates.

pub fn check(tool: &str, output: &str) -> Result<(), String> {
    match tool {
        "bash" => {
            let exit_line = output.lines().find(|l| l.starts_with("exit: "));
            match exit_line {
                None => Err("bash result missing exit-code line".into()),
                Some(l) => {
                    let code: i32 = l
                        .trim_start_matches("exit: ")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .parse()
                        .map_err(|_| "bash exit code not a number")?;
                    if !(-128..=255).contains(&code) {
                        return Err(format!("implausible bash exit code {code}"));
                    }
                    Ok(())
                }
            }
        }
        "write_file" => {
            if output.starts_with("wrote ") && output.contains(" bytes to ") {
                Ok(())
            } else {
                Err("write_file result malformed".into())
            }
        }
        "edit_file" => {
            if output.starts_with("edited ") {
                Ok(())
            } else {
                Err("edit_file result malformed".into())
            }
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_exit_codes_validated() {
        assert!(check("bash", "exit: 0\nstdout:\nhi").is_ok());
        assert!(check("bash", "exit: 255\nstdout:\n").is_ok());
        assert!(check("bash", "garbage without exit line").is_err());
        assert!(check("bash", "exit: 9999\n").is_err());
    }

    #[test]
    fn write_results_validated() {
        assert!(check("write_file", "wrote 12 bytes to f.txt").is_ok());
        assert!(check("write_file", "something else").is_err());
    }
}
