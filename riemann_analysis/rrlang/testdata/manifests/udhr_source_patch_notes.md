# UDHR source patch notes

The original pack tried Unicode direct text URLs and OHCHR search pages. Those now fail for two reasons:

1. Unicode stopped hosting the UDHR in Unicode project directly as of January 2024.
2. OHCHR search endpoints can block scripted access even though the translations are publicly listed.

This patch uses the NLTK data archive packages `udhr2.zip` and fallback `udhr.zip` from the NLTK GitHub data distribution.
