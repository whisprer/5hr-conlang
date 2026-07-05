use rrlang_core::{
    inspect_file, load_config_file, report_to_json, report_to_text, run_analysis, AnalyseOptions,
    CasePolicy, EncodingKind, HyphenPolicy, PunctuationPolicy, Result, RrlangError, WhitespacePolicy,
};
use std::env;
use std::fs;

fn main() {
    if let Err(err) = run() {
        eprintln!("rrlang error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    let command = args.remove(0);
    match command.as_str() {
        "inspect" => command_inspect(&args),
        "analyse" | "analyze" => command_analyse(&args),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(RrlangError::Message(format!(
            "Unknown command '{other}'. Run 'rrlang help'."
        ))),
    }
}

fn command_inspect(args: &[String]) -> Result<()> {
    let input = get_arg_value(args, "--input")
        .or_else(|| get_arg_value(args, "-i"))
        .ok_or_else(|| RrlangError::Message("inspect requires --input <PATH>".to_string()))?;
    let inspection = inspect_file(&input)?;
    println!("RRLANG CORPUS INSPECTION");
    println!("=========================");
    println!("path: {}", inspection.path);
    println!("bytes: {}", inspection.byte_len);
    println!("chars: {}", inspection.char_len);
    println!("lines: {}", inspection.line_count);
    println!("word_like_tokens: {}", inspection.word_like_count);
    println!("unique_word_like_tokens: {}", inspection.unique_word_like_count);
    Ok(())
}

fn command_analyse(args: &[String]) -> Result<()> {
    let mut options = if let Some(config_path) = get_arg_value(args, "--config") {
        load_config_file(&config_path)?
    } else {
        AnalyseOptions::default()
    };

    apply_cli_overrides(args, &mut options)?;

    if options.input_path.trim().is_empty() {
        return Err(RrlangError::Message(
            "analyse requires --input <PATH> or input_path in --config.".to_string(),
        ));
    }

    let report = run_analysis(&options)?;
    let json = report_to_json(&report);
    let text = report_to_text(&report);

    if let Some(path) = &options.output_json {
        fs::write(path, json)?;
        println!("Wrote JSON report: {path}");
    } else {
        println!("{json}");
    }

    if let Some(path) = &options.output_text {
        fs::write(path, text)?;
        println!("Wrote text report: {path}");
    }

    Ok(())
}

fn apply_cli_overrides(args: &[String], options: &mut AnalyseOptions) -> Result<()> {
    if let Some(input) = get_arg_value(args, "--input").or_else(|| get_arg_value(args, "-i")) {
        options.input_path = input;
    }
    if let Some(language) = get_arg_value(args, "--language") {
        options.language = language;
    }
    if let Some(name) = get_arg_value(args, "--name") {
        options.experiment_name = name;
    }
    if let Some(output_json) = get_arg_value(args, "--out").or_else(|| get_arg_value(args, "--json")) {
        options.output_json = if output_json.eq_ignore_ascii_case("none") { None } else { Some(output_json) };
    }
    if let Some(output_text) = get_arg_value(args, "--text-out").or_else(|| get_arg_value(args, "--txt")) {
        options.output_text = if output_text.eq_ignore_ascii_case("none") { None } else { Some(output_text) };
    }
    if let Some(nulls) = get_arg_value(args, "--nulls") {
        options.null_samples = nulls.parse::<usize>().map_err(|_| {
            RrlangError::Message(format!("Invalid --nulls value: {nulls}"))
        })?;
    }
    if let Some(seed) = get_arg_value(args, "--seed") {
        options.seed = seed.parse::<u64>().map_err(|_| {
            RrlangError::Message(format!("Invalid --seed value: {seed}"))
        })?;
    }
    if let Some(encodings) = get_arg_value(args, "--encodings").or_else(|| get_arg_value(args, "--encoding")) {
        let parsed = parse_encoding_list(&encodings)?;
        if !parsed.is_empty() {
            options.encodings = parsed;
        }
    }
    if let Some(case_policy) = get_arg_value(args, "--case-policy") {
        options.case_policy = CasePolicy::from_name(&case_policy).ok_or_else(|| {
            RrlangError::Message(format!("Unknown --case-policy value: {case_policy}"))
        })?;
    }
    if let Some(punctuation_policy) = get_arg_value(args, "--punctuation-policy") {
        options.punctuation_policy = PunctuationPolicy::from_name(&punctuation_policy).ok_or_else(|| {
            RrlangError::Message(format!("Unknown --punctuation-policy value: {punctuation_policy}"))
        })?;
    }

    if let Some(hyphen_policy) = get_arg_value(args, "--hyphen-policy") {
        options.hyphen_policy = HyphenPolicy::from_name(&hyphen_policy).ok_or_else(|| {
            RrlangError::Message(format!("Unknown --hyphen-policy value: {hyphen_policy}"))
        })?;
    }
    if let Some(whitespace_policy) = get_arg_value(args, "--whitespace-policy") {
        options.whitespace_policy = WhitespacePolicy::from_name(&whitespace_policy).ok_or_else(|| {
            RrlangError::Message(format!("Unknown --whitespace-policy value: {whitespace_policy}"))
        })?;
    }
    Ok(())
}

fn parse_encoding_list(input: &str) -> Result<Vec<EncodingKind>> {
    let mut encodings = Vec::new();
    for raw_name in input.split(',') {
        let name = raw_name.trim();
        if name.is_empty() {
            continue;
        }
        let kind = EncodingKind::from_name(name)
            .ok_or_else(|| RrlangError::Message(format!("Unknown encoding: {name}")))?;
        encodings.push(kind);
    }
    Ok(encodings)
}

fn get_arg_value(args: &[String], key: &str) -> Option<String> {
    for index in 0..args.len() {
        if args[index] == key {
            return args.get(index + 1).cloned();
        }
        if let Some(rest) = args[index].strip_prefix(&format!("{key}=")) {
            return Some(rest.to_string());
        }
    }
    None
}

fn print_help() {
    println!(
        "{}",
        r#"rrlang 0.2.0
Riemann-Resonant Linguistics research instrument MVP.

USAGE:
  rrlang inspect --input <PATH>
  rrlang analyse --input <PATH> [OPTIONS]
  rrlang analyse --config <PATH> [OPTIONS]

COMMANDS:
  inspect      Print basic corpus size/token information.
  analyse      Run MVP encoding, metric, null-model, and alert pipeline.
  help         Show this help.

ANALYSE OPTIONS:
  --input, -i <PATH>                 Input UTF-8 text file.
  --config <PATH>                    Optional simple TOML-like config file.
  --language <NAME>                  Language label for the report.
  --name <NAME>                      Experiment name.
  --encodings <LIST>                 Comma list: utf8_bits,bit_text,grapheme,grapheme_class,word_boundary,frequency_class
  --nulls <N>                        Number of null samples per null model. Default: 100.
  --seed <N>                         Deterministic RNG seed. Default: 18427.
  --out <PATH|none>                  JSON report path. Default: rrlang_report.json.
  --text-out <PATH|none>             Text report path. Default: rrlang_report.txt.
  --case-policy <preserve|lowercase> Default: lowercase.
  --punctuation-policy <preserve|remove>
  --hyphen-policy <punctuation|morpheme_boundary|word_internal|remove>
  --whitespace-policy <preserve|normalise>

EXAMPLES:
  rrlang inspect --input testdata/tiny/english.txt
  rrlang analyse --input testdata/tiny/english.txt --language English --nulls 100 --out outputs/english.json --text-out outputs/english.txt
  rrlang analyse --config examples/config_basic.toml

INTERPRETATION:
  rrlang reports measurements and evidence-tiered warnings. It does not classify origin,
  prove artificiality, or make Riemann-hypothesis claims.
"#
    );
}
