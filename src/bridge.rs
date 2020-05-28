use clap::Clap;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use copyrat::{error, process, CliOpt};

mod tmux;

trait Executor {
    fn execute(&mut self, args: Vec<String>) -> String;
    fn last_executed(&self) -> Option<Vec<String>>;
}

struct RealShell {
    executed: Option<Vec<String>>,
}

impl RealShell {
    fn new() -> RealShell {
        RealShell { executed: None }
    }
}

impl Executor for RealShell {
    fn execute(&mut self, args: Vec<String>) -> String {
        let execution = Command::new(args[0].as_str())
            .args(&args[1..])
            .output()
            .expect("Execution failed");

        self.executed = Some(args);

        let output: String = String::from_utf8_lossy(&execution.stdout).into();

        output.trim_end().to_string()
    }

    fn last_executed(&self) -> Option<Vec<String>> {
        self.executed.clone()
    }
}

const TMP_FILE: &str = "/tmp/copyrat-last";

pub struct Swapper<'a> {
    executor: Box<&'a mut dyn Executor>,
    // directory: &'a path::Path,
    command: &'a str,
    alt_command: &'a str,
    active_pane_id: Option<String>,
    active_pane_height: Option<i32>,
    active_pane_scroll_position: Option<i32>,
    active_pane_in_copy_mode: Option<String>,
    thumbs_pane_id: Option<String>,
    content: Option<String>,
    signal: String,
}

impl<'a> Swapper<'a> {
    fn new(
        executor: Box<&'a mut dyn Executor>,
        // directory: &'a path::Path,
        command: &'a str,
        alt_command: &'a str,
    ) -> Swapper<'a> {
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let signal = format!("thumbs-finished-{}", since_the_epoch.as_secs());

        Swapper {
            executor,
            // directory,
            command,
            alt_command,
            active_pane_id: None,
            active_pane_height: None,
            active_pane_scroll_position: None,
            active_pane_in_copy_mode: None,
            thumbs_pane_id: None,
            content: None,
            signal,
        }
    }

    pub fn capture_active_pane(&mut self) {
        let active_command = vec![
            "tmux",
            "list-panes",
            "-F",
            "#{pane_id}:#{?pane_in_mode,1,0}:#{pane_height}:#{scroll_position}:#{?pane_active,active,nope}",
        ];

        let output = self
            .executor
            .execute(active_command.iter().map(|arg| arg.to_string()).collect());

        let lines: Vec<&str> = output.split('\n').collect();
        let chunks: Vec<Vec<&str>> = lines
            .into_iter()
            .map(|line| line.split(':').collect())
            .collect();

        let active_pane = chunks
            .iter()
            .find(|&chunks| *chunks.get(4).unwrap() == "active")
            .expect("Unable to find active pane");

        let pane_id = active_pane.get(0).unwrap();
        let pane_in_copy_mode = active_pane.get(1).unwrap().to_string();

        self.active_pane_id = Some(pane_id.to_string());
        self.active_pane_in_copy_mode = Some(pane_in_copy_mode);

        if self.active_pane_in_copy_mode.clone().unwrap() == "1" {
            let pane_height = active_pane
                .get(2)
                .unwrap()
                .parse()
                .expect("Unable to retrieve pane height");
            let pane_scroll_position = active_pane
                .get(3)
                .unwrap()
                .parse()
                .expect("Unable to retrieve pane scroll");

            self.active_pane_height = Some(pane_height);
            self.active_pane_scroll_position = Some(pane_scroll_position);
        }
    }

    pub fn execute_thumbs(&mut self) {
        let options_command = vec!["tmux", "show", "-g"];
        let params: Vec<String> = options_command.iter().map(|arg| arg.to_string()).collect();
        let options = self.executor.execute(params);
        let lines: Vec<&str> = options.split('\n').collect();

        let pattern = Regex::new(r#"@thumbs-([\w\-0-9]+) "?(\w+)"?"#).unwrap();

        let args = lines
            .iter()
            .flat_map(|line| {
                if let Some(captures) = pattern.captures(line) {
                    let name = captures.get(1).unwrap().as_str();
                    let value = captures.get(2).unwrap().as_str();

                    let boolean_params = vec!["reverse", "unique", "contrast"];

                    if boolean_params.iter().any(|&x| x == name) {
                        return vec![format!("--{}", name)];
                    }

                    let string_params = vec![
                        "position",
                        "fg-color",
                        "bg-color",
                        "hint-bg-color",
                        "hint-fg-color",
                        "select-fg-color",
                        "select-bg-color",
                    ];

                    if string_params.iter().any(|&x| x == name) {
                        return vec![format!("--{}", name), format!("'{}'", value)];
                    }

                    if name.starts_with("regexp") {
                        return vec!["--regexp".to_string(), format!("'{}'", value)];
                    }

                    vec![]
                } else {
                    vec![]
                }
            })
            .collect::<Vec<String>>();

        let active_pane_id = self.active_pane_id.as_mut().unwrap().clone();

        let scroll_params = if self.active_pane_in_copy_mode.is_some() {
            if let (Some(pane_height), Some(scroll_position)) = (
                self.active_pane_scroll_position,
                self.active_pane_scroll_position,
            ) {
                format!(
                    " -S {} -E {}",
                    -scroll_position,
                    pane_height - scroll_position - 1
                )
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // NOTE: For debugging add echo $PWD && sleep 5 after tee
        let pane_command = format!(
            "tmux capture-pane -t {active_id} -p{scroll_params} | target/release/thumbs -f '%U:%H' -t {tmpfile} {args}; tmux swap-pane -t {active_id}; tmux wait-for -S {signal}",
            active_id = active_pane_id,
            scroll_params = scroll_params,
            // dir = self.directory.to_str().unwrap(),
            tmpfile = TMP_FILE,
            args = args.join(" "),
            signal = self.signal
        );

        let thumbs_command = vec![
            "tmux",
            "new-window",
            "-P",
            "-d",
            "-n",
            "[thumbs]",
            pane_command.as_str(),
        ];

        let params: Vec<String> = thumbs_command.iter().map(|arg| arg.to_string()).collect();

        self.thumbs_pane_id = Some(self.executor.execute(params));
    }

    pub fn swap_panes(&mut self) {
        let active_pane_id = self.active_pane_id.as_mut().unwrap().clone();
        let thumbs_pane_id = self.thumbs_pane_id.as_mut().unwrap().clone();

        let swap_command = vec![
            "tmux",
            "swap-pane",
            "-d",
            "-s",
            active_pane_id.as_str(),
            "-t",
            thumbs_pane_id.as_str(),
        ];
        let params = swap_command.iter().map(|arg| arg.to_string()).collect();

        self.executor.execute(params);
    }

    pub fn wait_thumbs(&mut self) {
        let wait_command = vec!["tmux", "wait-for", self.signal.as_str()];
        let params = wait_command.iter().map(|arg| arg.to_string()).collect();

        self.executor.execute(params);
    }

    pub fn retrieve_content(&mut self) {
        let retrieve_command = vec!["cat", TMP_FILE];
        let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

        self.content = Some(self.executor.execute(params));
    }

    pub fn destroy_content(&mut self) {
        let retrieve_command = vec!["rm", TMP_FILE];
        let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

        self.executor.execute(params);
    }

    pub fn execute_command(&mut self) {
        let content = self.content.clone().unwrap();
        let mut splitter = content.splitn(2, ':');

        if let Some(upcase) = splitter.next() {
            if let Some(text) = splitter.next() {
                let execute_command = if upcase.trim_end() == "true" {
                    self.alt_command.clone()
                } else {
                    self.command.clone()
                };

                let final_command = str::replace(execute_command, "{}", text.trim_end());
                let retrieve_command = vec!["bash", "-c", final_command.as_str()];
                let params = retrieve_command.iter().map(|arg| arg.to_string()).collect();

                self.executor.execute(params);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestShell {
        outputs: Vec<String>,
        executed: Option<Vec<String>>,
    }

    impl TestShell {
        fn new(outputs: Vec<String>) -> TestShell {
            TestShell {
                executed: None,
                outputs,
            }
        }
    }

    impl Executor for TestShell {
        fn execute(&mut self, args: Vec<String>) -> String {
            self.executed = Some(args);
            self.outputs.pop().unwrap()
        }

        fn last_executed(&self) -> Option<Vec<String>> {
            self.executed.clone()
        }
    }

    #[test]
    fn retrieve_active_pane() {
        let last_command_outputs =
            vec!["%97:100:24:1:active\n%106:100:24:1:nope\n%107:100:24:1:nope\n".to_string()];
        let mut executor = TestShell::new(last_command_outputs);
        let mut swapper = Swapper::new(Box::new(&mut executor), "", "");

        swapper.capture_active_pane();

        assert_eq!(swapper.active_pane_id.unwrap(), "%97");
    }

    #[test]
    fn swap_panes() {
        let last_command_outputs = vec![
            "".to_string(),
            "%100".to_string(),
            "".to_string(),
            "%106:100:24:1:nope\n%98:100:24:1:active\n%107:100:24:1:nope\n".to_string(),
        ];
        let mut executor = TestShell::new(last_command_outputs);
        let mut swapper = Swapper::new(Box::new(&mut executor), "", "");

        swapper.capture_active_pane();
        swapper.execute_thumbs();
        swapper.swap_panes();

        let expectation = vec!["tmux", "swap-pane", "-d", "-s", "%98", "-t", "%100"];

        assert_eq!(executor.last_executed().unwrap(), expectation);
    }
}

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct BridgeOpt {
    /// Command to execute on selection.
    #[clap(long, default_value = "tmux set-buffer {}")]
    command: String,

    /// Command to execute on uppercased selection.
    #[clap(long, default_value = "tmux set-buffer {} && tmux-paste-buffer")]
    alt_command: String,

    /// Retrieve options from tmux.
    ///
    /// If active, options formatted like `copyrat-*` are read from tmux.
    /// You should consider reading them from the config file (the default
    /// option) as this saves both a command call (about 10ms) and a Regex
    /// compilation.
    #[clap(short = "T", long)]
    get_options_from_tmux: bool,

    /// Optionally capture entire pane history.
    #[clap(long, arg_enum, default_value = "entire-history")]
    capture: tmux::CaptureRegion,

    // Include CLI Options
    #[clap(flatten)]
    cli_options: CliOpt,
}

impl BridgeOpt {
    /// Try parsing provided options, and update self with the valid values.
    pub fn merge_map(
        &mut self,
        options: &HashMap<String, String>,
    ) -> Result<(), error::ParseError> {
        for (name, value) in options {
            match name.as_ref() {
                "@copyrat-command" => {
                    self.command = String::from(value);
                }
                "@copyrat-alt-command" => {
                    self.alt_command = String::from(value);
                }
                "@copyrat-capture" => {
                    self.capture = tmux::CaptureRegion::from_str(&value)?;
                }

                // Ignore unknown options.
                _ => (),
            }
        }

        // Pass the call to cli_options.
        self.cli_options.merge_map(options)?;

        Ok(())
    }
}

fn main() -> Result<(), error::ParseError> {
    let mut opt = BridgeOpt::parse();

    if opt.get_options_from_tmux {
        let tmux_options = tmux::get_options("@copyrat-")?;
        opt.merge_map(&tmux_options)?;
    }

    let panes: Vec<tmux::Pane> = tmux::list_panes()?;

    let active_pane = panes
        .iter()
        .find(|p| p.is_active)
        .expect("One tmux pane should be active");

    let buffer = tmux::capture_pane(&active_pane, &opt.capture)?;

    let selections: Vec<(String, bool)> = if active_pane.in_mode {
        // If the current pane is in copy mode, we have to dance a little with
        // Panes, because the current pane has already locked the Alternate
        // Screen, preventing copyrat::run to execute.
        let initial_pane = active_pane;

        // Create a new window without switching to it, with a `sh` command
        // for faster startup.
        let temp_pane: tmux::Pane = tmux::create_new_window("[copyrat]", "sh")?;

        // Swap the two panes, changing the active pane to be the temp_pane.
        // After swap, temp_pane has the same height than the initial_pane
        // had before being swapped.
        tmux::swap_panes(initial_pane, &temp_pane)?;

        // Running copyrat now will render in the newly created temp_pane
        // (locking stdin, writing to its stdout), but this is almost
        // transparent to the user.
        let selections = copyrat::run(buffer, &opt.cli_options);

        // Swap back the two panes, making initial_pane the active one again.
        tmux::swap_panes(&temp_pane, initial_pane)?;

        tmux::kill_pane(&temp_pane)?;

        selections
    } else {
        copyrat::run(buffer, &opt.cli_options)
    };

    // Execute a command on each selection.
    // TODO: consider getting rid of multi-selection mode.
    selections.iter().for_each(|(text, uppercased)| {
        let raw_command = if *uppercased {
            opt.alt_command.replace("{}", text)
        } else {
            opt.command.replace("{}", text)
        };
        let mut it = raw_command.split(' ').into_iter();
        let command = it.next().unwrap();
        let args: Vec<&str> = it.collect();

        // Simply execute the command as is, and let the program crash on
        // potential errors because it is not our responsibility.
        process::execute(&command, &args).unwrap();
    });

    if false {
        let mut executor = RealShell::new();

        let mut swapper = Swapper::new(
            Box::new(&mut executor),
            // opt.directory.as_path(),
            &opt.command,
            &opt.alt_command,
        );

        swapper.capture_active_pane();
        swapper.execute_thumbs();
        swapper.swap_panes();
        swapper.wait_thumbs();
        swapper.retrieve_content();
        swapper.destroy_content();
        swapper.execute_command();
    }

    Ok(())
}
