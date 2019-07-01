use std::io::Error;
use std::process::Command;

pub struct Executor {
    executable: String,
    arguments: Vec<String>,
}

impl Executor {
    pub fn new(command_and_args: &Vec<String>) -> Executor {
        let (exec, args) = command_and_args.split_first().unwrap();

        Executor {
            executable: exec.to_string(),
            arguments: args.to_vec(),
        }
    }

    pub fn execute(&self) -> Result<(), Error> {
        let output = Command::new(&self.executable)
                        .args(&self.arguments)
                        .output()?;

        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}
