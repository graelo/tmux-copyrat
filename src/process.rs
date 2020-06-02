use std::process::Command;

use crate::error::ParseError;

/// Execute an arbitrary Unix command and return the stdout as a `String` if
/// successful.
pub fn execute(command: &str, args: &Vec<&str>) -> Result<String, ParseError> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        return Err(ParseError::ProcessFailure(format!(
            "Process failure: {} {}, error {}",
            command,
            args.join(" "),
            msg
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
