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

pub(crate) const SIPS_COMMAND: &str = "sips";
pub(crate) const DEREZ_COMMAND: &str = "DeRez";
pub(crate) const REZ_COMMAND: &str = "Rez";
pub(crate) const SETFILE_COMMAND: &str = "SetFile";

pub(crate) fn run_command(
    command_name: &str,
    args: &CommandArgs,
) -> Result<Vec<u8>, FolderifyError> {
    if DEBUG_PRINT_ARGS {
        println!("args: {}", args.args.join(" "));
    };
    let child = Command::new(command_name)
        .args(args.args.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let child = match child {
        Ok(child) => child,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };

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

pub(crate) fn run_convert(args: &CommandArgs) -> Result<(), FolderifyError> {
    run_command(CONVERT_COMMAND, args)?;
    Ok(())
}

pub(crate) fn identify_read_u32(args: &CommandArgs) -> Result<u32, FolderifyError> {
    let stdout = run_command(IDENTIFY_COMMAND, args)?;
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
