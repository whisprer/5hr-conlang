# Source notes for v0.3 corpus harvester

- UDHR: fetched from NLTK UDHR/UDHR2 package mirrors on GitHub. Use UN/OHCHR pages for final legal/publication verification.
- Tatoeba: default uses `sentences_CC0.tar.bz2`, the CC0 subset, to avoid attribution complexity for early tests.
- Wikipedia: fetched via MediaWiki API random/extract requests. Treat as CC BY-SA content and preserve attribution if redistributing.
- FLORES-200: optional via Python `datasets`; install with `py -3 -m pip install datasets` if needed.
- Gutenberg: optional curated IDs only; verify local public-domain status and Gutenberg license terms before redistribution.

This tool deliberately avoids default Wikimedia dumps and OPUS bulk downloads because those can rapidly become hundreds of MB to many GB per source. Add them only once the smaller corpora produce stable results.
