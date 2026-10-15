//! The argument parser. Each command declares its flags, and an unknown flag is an error.

#[derive(Clone, Copy)]
pub struct Flag {
    pub long: &'static str,
    pub short: Option<char>,
    pub takes_value: bool,
}

pub const fn switch(long: &'static str, short: Option<char>) -> Flag {
    Flag {
        long,
        short,
        takes_value: false,
    }
}

pub const fn option(long: &'static str, short: Option<char>) -> Flag {
    Flag {
        long,
        short,
        takes_value: true,
    }
}

#[derive(Debug, Default)]
pub struct Args {
    pub positional: Vec<String>,
    flags: Vec<(String, Option<String>)>,
}

impl Args {
    pub fn has(&self, long: &str) -> bool {
        self.flags.iter().any(|(name, _)| name == long)
    }

    pub fn value(&self, long: &str) -> Option<&str> {
        self.flags
            .iter()
            .rev()
            .find(|(name, _)| name == long)
            .and_then(|(_, value)| value.as_deref())
    }

    pub fn values(&self, long: &str) -> Vec<String> {
        self.flags
            .iter()
            .filter(|(name, _)| name == long)
            .filter_map(|(_, value)| value.clone())
            .collect()
    }

    pub fn number(&self, long: &str) -> Result<Option<usize>, String> {
        match self.value(long) {
            None => Ok(None),
            Some(text) => text
                .parse::<usize>()
                .map(Some)
                .map_err(|_| format!("--{long} needs a number, not \"{text}\"")),
        }
    }

    pub fn expect_positional(&self, min: usize, max: usize, usage: &str) -> Result<(), String> {
        if self.positional.len() < min || self.positional.len() > max {
            return Err(format!("use: {usage}"));
        }
        Ok(())
    }
}

pub const HELP: Flag = switch("help", Some('h'));

pub fn parse(raw: &[String], flags: &[Flag], command: &str) -> Result<Args, String> {
    let mut args = Args::default();
    let mut index = 0;
    let mut only_positional = false;
    let find_long = |name: &str| flags.iter().chain([&HELP]).find(|flag| flag.long == name);
    let find_short = |c: char| {
        flags
            .iter()
            .chain([&HELP])
            .find(|flag| flag.short == Some(c))
    };
    let unknown = |text: &str| {
        format!("unknown option {text} for \"todoer {command}\" - run todoer {command} --help")
    };
    while index < raw.len() {
        let item = &raw[index];
        index += 1;
        if only_positional || item == "-" || !item.starts_with('-') {
            args.positional.push(item.clone());
            continue;
        }
        if item == "--" {
            only_positional = true;
            continue;
        }
        let (flag, inline) = if let Some(long) = item.strip_prefix("--") {
            let (name, inline) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (long, None),
            };
            (find_long(name).ok_or_else(|| unknown(item))?, inline)
        } else {
            let mut chars = item[1..].chars();
            let first = chars.next().unwrap_or('-');
            let rest: String = chars.collect();
            let flag = find_short(first).ok_or_else(|| unknown(item))?;
            if !flag.takes_value && !rest.is_empty() {
                for c in rest.chars() {
                    let other = find_short(c).ok_or_else(|| unknown(item))?;
                    if other.takes_value {
                        return Err(format!("-{c} needs a value - write it on its own"));
                    }
                    args.flags.push((other.long.to_string(), None));
                }
                (flag, None)
            } else {
                (flag, (!rest.is_empty()).then_some(rest))
            }
        };
        if flag.takes_value {
            let value = match inline {
                Some(value) => value,
                None => {
                    let Some(value) = raw.get(index) else {
                        return Err(format!("--{} needs a value", flag.long));
                    };
                    index += 1;
                    value.clone()
                }
            };
            args.flags.push((flag.long.to_string(), Some(value)));
        } else {
            if inline.is_some() {
                return Err(format!("--{} takes no value", flag.long));
            }
            args.flags.push((flag.long.to_string(), None));
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    const FLAGS: &[Flag] = &[
        option("title", Some('t')),
        option("label", Some('l')),
        switch("all", Some('a')),
        switch("json", None),
    ];

    #[test]
    fn parses_flags_and_positional() {
        let args = parse(
            &strings(&["-t", "A title", "--label=x", "-l", "y", "text", "--json"]),
            FLAGS,
            "add",
        )
        .unwrap();
        assert_eq!(args.value("title"), Some("A title"));
        assert_eq!(args.values("label"), vec!["x", "y"]);
        assert!(args.has("json"));
        assert_eq!(args.positional, vec!["text"]);
    }

    #[test]
    fn stops_at_double_dash() {
        let args = parse(&strings(&["--", "-t", "x"]), FLAGS, "add").unwrap();
        assert_eq!(args.positional, vec!["-t", "x"]);
    }

    #[test]
    fn refuses_unknown_flags() {
        let error = parse(&strings(&["--nope"]), FLAGS, "add").unwrap_err();
        assert!(error.starts_with("unknown option --nope"));
        assert!(parse(&strings(&["-t"]), FLAGS, "add").is_err());
        assert!(parse(&strings(&["--all=1"]), FLAGS, "add").is_err());
    }

    #[test]
    fn accepts_short_values_and_groups() {
        let args = parse(&strings(&["-tTitle", "-ah"]), FLAGS, "add").unwrap();
        assert_eq!(args.value("title"), Some("Title"));
        assert!(args.has("all"));
        assert!(args.has("help"));
    }
}
