use rrlang_core::{
    inspect_file, load_config_file, report_to_json, report_to_text, run_analysis, AnalyseOptions,
    CasePolicy, EncodingKind, HyphenPolicy, PunctuationPolicy, Result, RrlangError, WhitespacePolicy,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
        "batch" => command_batch(&args),
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

    if has_flag(args, "--progress") {
        println!(
            "RUN {} | encodings={} | nulls={} | max_chars={}",
            options.input_path,
            encoding_list_label(&options.encodings),
            options.null_samples,
            options
                .max_chars
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }

    let started = Instant::now();
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

    if has_flag(args, "--progress") {
        println!("DONE in {:.2}s", started.elapsed().as_secs_f64());
    }

    Ok(())
}

fn command_batch(args: &[String]) -> Result<()> {
    let dataset_root = get_arg_value(args, "--dataset-root")
        .or_else(|| get_arg_value(args, "--input-root"))
        .ok_or_else(|| RrlangError::Message("batch requires --dataset-root <DIR>".to_string()))?;
    let out_dir = get_arg_value(args, "--out-dir")
        .or_else(|| get_arg_value(args, "--outputs"))
        .ok_or_else(|| RrlangError::Message("batch requires --out-dir <DIR>".to_string()))?;

    let dataset_root_path = PathBuf::from(&dataset_root);
    let out_dir_path = PathBuf::from(&out_dir);
    fs::create_dir_all(&out_dir_path)?;

    let mut base_options = if let Some(config_path) = get_arg_value(args, "--config") {
        load_config_file(&config_path)?
    } else {
        AnalyseOptions::default()
    };
    apply_cli_overrides(args, &mut base_options)?;

    let skip_existing = has_flag(args, "--skip-existing") || has_flag(args, "--resume");
    let continue_on_error = has_flag(args, "--continue-on-error");
    let include_raw_inputs = has_flag(args, "--include-raw-inputs");

    let mut files = Vec::new();
    collect_txt_files(&dataset_root_path, &mut files)?;
    files.sort();

    if !include_raw_inputs {
        files.retain(|path| !is_skipped_input_path(path));
    }

    println!("RRLANG BATCH RUN");
    println!("================");
    println!("dataset_root: {}", dataset_root_path.display());
    println!("out_dir: {}", out_dir_path.display());
    println!("inputs: {}", files.len());
    println!("encodings: {}", encoding_list_label(&base_options.encodings));
    println!("nulls: {}", base_options.null_samples);
    println!(
        "max_chars: {}",
        base_options
            .max_chars
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("skip_existing: {}", skip_existing);
    println!();

    let mut completed = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for (index, input_path) in files.iter().enumerate() {
        let output_base = output_base_for(&dataset_root_path, input_path);
        let json_path = out_dir_path.join(format!("{output_base}.json"));
        let text_path = out_dir_path.join(format!("{output_base}.txt"));

        if skip_existing && json_path.exists() && text_path.exists() {
            println!("SKIP {}/{} {}", index + 1, files.len(), output_base);
            skipped += 1;
            continue;
        }

        println!("RUN  {}/{} {}", index + 1, files.len(), output_base);
        let started = Instant::now();

        let mut options = base_options.clone();
        options.input_path = input_path.to_string_lossy().to_string();
        options.output_json = Some(json_path.to_string_lossy().to_string());
        options.output_text = Some(text_path.to_string_lossy().to_string());

        match run_single_to_files(&options) {
            Ok(()) => {
                println!("OK   {} ({:.2}s)", output_base, started.elapsed().as_secs_f64());
                completed += 1;
            }
            Err(err) => {
                eprintln!("FAIL {}: {}", output_base, err);
                failed += 1;
                if !continue_on_error {
                    return Err(err);
                }
            }
        }
    }

    println!();
    println!("Batch summary:");
    println!("  completed: {}", completed);
    println!("  skipped:   {}", skipped);
    println!("  failed:    {}", failed);
    println!("  total:     {}", files.len());

    if failed > 0 {
        return Err(RrlangError::Message(format!(
            "Batch completed with {failed} failed input(s)."
        )));
    }

    Ok(())
}

fn run_single_to_files(options: &AnalyseOptions) -> Result<()> {
    let report = run_analysis(options)?;
    let json = report_to_json(&report);
    let text = report_to_text(&report);

    if let Some(path) = &options.output_json {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, json)?;
    }

    if let Some(path) = &options.output_text {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, text)?;
    }

    Ok(())
}

fn apply_cli_overrides(args: &[String], options: &mut AnalyseOptions) -> Result<()> {
    if has_flag(args, "--fast-profile") {
        options.null_samples = 25;
        options.encodings = linguistic_encodings();
        options.max_chars = Some(25_000);
    }

    if has_flag(args, "--linguistic-profile") {
        options.encodings = linguistic_encodings();
    }

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
        options.output_json = if output_json.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(output_json)
        };
    }
    if let Some(output_text) = get_arg_value(args, "--text-out").or_else(|| get_arg_value(args, "--txt")) {
        options.output_text = if output_text.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(output_text)
        };
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
    if let Some(max_chars) = get_arg_value(args, "--max-chars").or_else(|| get_arg_value(args, "--max-characters")) {
        if max_chars.eq_ignore_ascii_case("none") || max_chars == "0" {
            options.max_chars = None;
        } else {
            options.max_chars = Some(max_chars.parse::<usize>().map_err(|_| {
                RrlangError::Message(format!("Invalid --max-chars value: {max_chars}"))
            })?);
        }
    }
    if let Some(encodings) = get_arg_value(args, "--encodings").or_else(|| get_arg_value(args, "--encoding")) {
        let parsed = parse_encoding_list(&encodings)?;
        if !parsed.is_empty() {
            options.encodings = parsed;
        }
    }
    if has_flag(args, "--skip-raw") || has_flag(args, "--no-raw") {
        options
            .encodings
            .retain(|kind| !matches!(kind, EncodingKind::Utf8Bits | EncodingKind::BitText));
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

    if options.encodings.is_empty() {
        return Err(RrlangError::Message(
            "No encodings enabled. Remove --skip-raw or provide --encodings.".to_string(),
        ));
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

fn linguistic_encodings() -> Vec<EncodingKind> {
    vec![
        EncodingKind::Grapheme,
        EncodingKind::GraphemeClass,
        EncodingKind::WordBoundary,
        EncodingKind::FrequencyClass,
    ]
}

fn encoding_list_label(encodings: &[EncodingKind]) -> String {
    encodings
        .iter()
        .map(|kind| kind.as_str())
        .collect::<Vec<_>>()
        .join(",")
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

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn collect_txt_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_txt_files(&path, out)?;
        } else if metadata.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("txt"))
                .unwrap_or(false)
        {
            out.push(path);
        }
    }
    Ok(())
}

fn is_skipped_input_path(path: &Path) -> bool {
    let lowered = path.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    lowered.contains("/source_raw/")
        || lowered.contains("/_cache/")
        || lowered.ends_with("_raw.txt")
}

fn output_base_for(root: &Path, input_path: &Path) -> String {
    let rel = input_path.strip_prefix(root).unwrap_or(input_path);
    let without_ext = rel.with_extension("");
    sanitize_output_name(&without_ext.to_string_lossy())
}

fn sanitize_output_name(input: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;

    for ch in input.chars() {
        let keep = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        if keep {
            out.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore {
            out.push('_');
            last_was_underscore = true;
        }
    }

    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "rrlang_report".to_string()
    } else {
        trimmed
    }
}

fn print_help() {
    println!(
        "{}",
        r#"rrlang 0.3.0
Riemann-Resonant Linguistics research instrument MVP.

USAGE:
  rrlang inspect --input <PATH>
  rrlang analyse --input <PATH> [OPTIONS]
  rrlang analyse --config <PATH> [OPTIONS]
  rrlang batch --dataset-root <DIR> --out-dir <DIR> [OPTIONS]

COMMANDS:
  inspect      Print basic corpus size/token information.
  analyse      Run encoding, metric, null-model, and alert pipeline on one file.
  batch        Recursively analyse .txt files with progress and resume support.
  help         Show this help.

ANALYSE OPTIONS:
  --input, -i <PATH>                 Input UTF-8 text file.
  --config <PATH>                    Optional simple TOML-like config file.
  --language <NAME>                  Language label for the report.
  --name <NAME>                      Experiment name.
  --encodings <LIST>                 Comma list: utf8_bits,bit_text,grapheme,grapheme_class,word_boundary,frequency_class
  --skip-raw                         Remove utf8_bits and bit_text from enabled encodings.
  --linguistic-profile               Use grapheme,grapheme_class,word_boundary,frequency_class.
  --fast-profile                     Use linguistic profile, --nulls 25, and --max-chars 25000 unless overridden.
  --max-chars <N|none>               Cap cleaned input before analysis. Useful for large corpora.
  --nulls <N>                        Number of null samples per null model. Default: 100.
  --seed <N>                         Deterministic RNG seed. Default: 18427.
  --progress                         Print start/end timing for single-file analysis.
  --out <PATH|none>                  JSON report path. Default: rrlang_report.json.
  --text-out <PATH|none>             Text report path. Default: rrlang_report.txt.
  --case-policy <preserve|lowercase> Default: lowercase.
  --punctuation-policy <preserve|remove>
  --hyphen-policy <punctuation|morpheme_boundary|word_internal|remove>
  --whitespace-policy <preserve|normalise>

BATCH OPTIONS:
  --dataset-root <DIR>               Root folder containing .txt datasets.
  --out-dir <DIR>                    Output folder for report pairs.
  --skip-existing, --resume          Do not rerun files with existing .json and .txt outputs.
  --continue-on-error                Keep going after a failed input.
  --include-raw-inputs               Include source_raw, _cache, and *_raw.txt files.
  Plus all analyse options except --input/--out/--text-out, which batch sets per file.

EXAMPLES:
  rrlang inspect --input testdata/tiny/english.txt
  rrlang analyse --input testdata/tiny/english.txt --language English --skip-raw --nulls 25 --out outputs/english.json --text-out outputs/english.txt
  rrlang batch --dataset-root testdata/datasets_canonical/parallel/udhr --out-dir outputs/canonical_udhr_v0_3 --linguistic-profile --nulls 100 --skip-existing
  rrlang batch --dataset-root testdata/datasets --out-dir outputs/broad_fast_v0_3 --fast-profile --skip-existing --continue-on-error

INTERPRETATION:
  rrlang reports measurements and evidence-tiered warnings. It does not classify origin,
  prove artificiality, or make Riemann-hypothesis claims.
"#
    );
}
