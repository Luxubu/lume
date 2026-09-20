# Review corpus

Programs written by an independent reviewer after milestone 14, without
knowledge of the compiler: nine realistic programs (`p*.lume`) and small
edge-case probes (`edge/*.lume`). At the time of the review, none of the
nine compiled as first written, and about 40% of all probes ended in a
leaked rustc error. See the consolidated review for the ranked findings.

These are not in `tests/run.sh` yet. They are the bar for milestones
15–17: every file here either runs correctly or fails with a Lume error.
