#!/usr/bin/env python3
from pathlib import Path
import random
root = Path(__file__).resolve().parents[1]
out = root / "datasets" / "controls"
out.mkdir(parents=True, exist_ok=True)
rng = random.Random(18427)
(out / "random_bits_10k.txt").write_text("".join(rng.choice("01") for _ in range(10000)) + "\n", encoding="utf-8")
code = """fn main() {
    let values = [1, 2, 3, 5, 8, 13];
    for value in values {
        if value % 2 == 0 {
            println!(\"even: {}\", value);
        } else {
            println!(\"odd: {}\", value);
        }
    }
}
"""
(out / "code_sample_rust.rs").write_text(code, encoding="utf-8")
mesu = root / "datasets" / "parallel" / "udhr" / "mesu" / "udhr_mesu_text_only.txt"
if mesu.exists():
    txt = mesu.read_text(encoding="utf-8")
    (out / "duplicated_udhr_mesu.txt").write_text((txt.strip() + "\n\n") * 3, encoding="utf-8")
    noisy = txt.replace(" ", " !!! ").replace("\n", " ???\n")
    (out / "punctuation_noise_udhr_mesu.txt").write_text(noisy, encoding="utf-8")
print(f"controls written to {out}")
