use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::str::from_utf8;

use crate::convert::CommandArgs;
use crate::error::CommandFailedError;
use crate::error::CommandInvalidError;
use crate::error::FolderifyError;
use crate::error::GeneralError;

const DEBUG_PRINT_ARGS: bool = false;

const CONVERT_COMMAND: &str = "convert";
const IDENTIFY_COMMAND: &str = "identify";
pub(crate) const ICONUTIL_COMMAND: &str = "iconutil";
pub(crate) const OPEN_COMMAND: &str = "open";

pub(crate) const OSASCRIPT_COMMAND: &str = "osascript";
pub(crate) const FILEICON_COMMAND: &str = "fileicon";

pub(crate) const SIPS_COMMAND: &str = "sips";
pub(crate) const DEREZ_COMMAND: &str = "DeRez";
pub(crate) const REZ_COMMAND: &str = "Rez";
pub(crate) const SETFILE_COMMAND: &str = "SetFile";

pub(crate) fn run_command(
    command_name: &str,
    args: &CommandArgs,
    stdin: Option<&[u8]>,
) -> Result<Vec<u8>, FolderifyError> {
    if DEBUG_PRINT_ARGS {
        println!("args: {}", args.args.join(" "));
    };
    let child = Command::new(command_name)
        .args(args.args.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };

    if let Some(stdin) = stdin {
        let child_stdin = child.stdin.as_mut().unwrap(); // TODO
        match child_stdin.write_all(stdin) {
            Ok(output) => output,
            Err(_) => {
                return Err(FolderifyError::General(GeneralError {
                    message: "Could not write to stdin for a command.".into(),
                }))
            }
        }
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }))
        }
    };

    if !output.status.success() {
        return Err(FolderifyError::CommandFailed(CommandFailedError {
            command_name: command_name.into(),
            stderr: output.stderr,
        }));
    }

    Ok(output.stdout)
}

pub(crate) fn run_convert(args: &CommandArgs, stdin: Option<&[u8]>) -> Result<(), FolderifyError> {
    run_command(CONVERT_COMMAND, args, stdin)?;
    Ok(())
}

pub(crate) fn identify_read_u32(args: &CommandArgs) -> Result<u32, FolderifyError> {
    let stdout = run_command(IDENTIFY_COMMAND, args, None)?;
    let s: &str = match from_utf8(&stdout) {
        Ok(s) => s,
        Err(s) => {
            println!("errerrerr{}+++++", s);
            return Err((GeneralError {
                message: "Could not read input dimensions".into(),
            })
            .into());
        }
    };
    let value = match s.parse::<u32>() {
        Ok(value) => value,
        Err(s) => {
            // TODO
            println!("errerrerr{}+++++", s);
            return Err((GeneralError {
                message: "Could not read input dimensions".into(),
            })
            .into());
        }
    };
    Ok(value)
}
