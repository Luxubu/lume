#!/usr/bin/env bash
# Packages from git, with no network: every repository is a local bare one
# made here, and the cache ($LUME_HOME) is a fresh folder. Prints what each
# step printed, with the temporary folder written as <tmp>, for
# tests/run.sh to compare with tests/git_packages.expected.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
tmp=$(cd "$(mktemp -d)" && pwd -P)
export LUME_HOME="$tmp/home"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
g() { git -c user.name=lume -c user.email=lume@example.com -c init.defaultBranch=main "$@"; }
step() { echo "--- $1"; }
# the cache names a checkout by a hash of its URL and the commit
run() { (cd "$1" && shift && "$LUME" "$@" 2>&1; echo "exit: $?") | sed -E "s#$tmp#<tmp>#g; s#checkouts/[^/]+/[0-9a-f]+/#checkouts/<package>/<commit>/#g"; }

# a library with two tagged versions, then one more commit on main
mkdir -p "$tmp/src/colors" && cd "$tmp/src/colors" || exit 1
g init -q .
printf '[package]\nname = "colors"\nversion = "0.1.0"\n' > lume.toml
printf 'pub def red -> Str = "red 1"\n' > lib.lume
g add -A && g commit -qm one && g tag v1
printf 'pub def red -> Str = "red 2"\n' > lib.lume
g commit -qam two && g tag v2
g clone -q --bare . "$tmp/colors.git"
v1=$(g rev-parse v1)

app() { # app <dir> <dependency line>
  mkdir -p "$tmp/$1"
  printf '[package]\nname = "app"\nversion = "0.1.0"\n\n[dependencies]\n%s\n' "$2" > "$tmp/$1/lume.toml"
  printf 'import colors\n\ndef main:\n  puts colors.red\n' > "$tmp/$1/main.lume"
}

step "a tag: fetched once, then locked"
app tagged 'colors = { git = "../colors.git", tag = "v1" }'
run "$tmp/tagged" run
sed "s#$tmp#<tmp>#g; s#$v1#<v1>#" "$tmp/tagged/lume.lock"
run "$tmp/tagged" run

step "the lock holds while the repository is gone: no network needed"
mv "$tmp/colors.git" "$tmp/away.git"
run "$tmp/tagged" run
mv "$tmp/away.git" "$tmp/colors.git"

step "a rev"
app byrev "colors = { git = \"../colors.git\", rev = \"${v1:0:10}\" }"
run "$tmp/byrev" run

step "the default branch moves; the lock does not, until lume update"
app onmain 'colors = { git = "../colors.git" }'
run "$tmp/onmain" run
cd "$tmp/src/colors" && printf 'pub def red -> Str = "red 3"\n' > lib.lume && g commit -qam three && g push -q "$tmp/colors.git" main
run "$tmp/onmain" run
run "$tmp/onmain" update | sed -E 's/[0-9a-f]{12}/<commit>/g'
run "$tmp/onmain" run
run "$tmp/onmain" update colors | sed -E 's/[0-9a-f]{12}/<commit>/g'

step "a tag that does not exist"
app notag 'colors = { git = "../colors.git", tag = "v9" }'
run "$tmp/notag" check

step "a repository that does not exist"
app norepo 'colors = { git = "../nothing.git", tag = "v1" }'
run "$tmp/norepo" check | sed -E 's/(cannot fetch `colors` from [^:]*):.*/\1: <git says why>/'

step "two packages asking for two commits of one package"
for n in a b; do
  mkdir -p "$tmp/src/$n" && cd "$tmp/src/$n" && g init -q .
  tag=$([ $n = a ] && echo v1 || echo v2)
  printf '[package]\nname = "%s"\nversion = "0.1.0"\n\n[dependencies]\ncolors = { git = "%s", tag = "%s" }\n' $n "$tmp/colors.git" $tag > lume.toml
  printf 'import colors\n\npub def shade -> Str = colors.red\n' > lib.lume
  g add -A && g commit -qm one && g clone -q --bare . "$tmp/$n.git"
done
mkdir -p "$tmp/both"
printf '[package]\nname = "both"\nversion = "0.1.0"\n\n[dependencies]\na = { git = "../a.git" }\nb = { git = "../b.git" }\n' > "$tmp/both/lume.toml"
printf 'import a\nimport b\n\ndef main:\n  puts a.shade\n' > "$tmp/both/main.lume"
run "$tmp/both" check | sed -E 's/[0-9a-f]{12}/<commit>/g'

step "a package from git that uses one from git: both locked"
mkdir -p "$tmp/one"
printf '[package]\nname = "one"\nversion = "0.1.0"\n\n[dependencies]\na = { git = "../a.git" }\n' > "$tmp/one/lume.toml"
printf 'import a\n\ndef main:\n  puts a.shade\n' > "$tmp/one/main.lume"
run "$tmp/one" run
grep '^name' "$tmp/one/lume.lock"

step "the same package from git and from a path"
mkdir -p "$tmp/mixed"
printf '[package]\nname = "mixed"\nversion = "0.1.0"\n\n[dependencies]\na = { git = "../a.git" }\ncolors = { path = "../src/colors" }\n' > "$tmp/mixed/lume.toml"
printf 'import a\n\ndef main:\n  puts a.shade\n' > "$tmp/mixed/main.lume"
run "$tmp/mixed" check

step "a repository holding packages in folders: the one named is found"
mkdir -p "$tmp/src/mono/packages/greet" "$tmp/src/mono/packages/other" && cd "$tmp/src/mono" || exit 1
g init -q .
printf '[package]\nname = "greet"\nversion = "0.1.0"\n' > packages/greet/lume.toml
printf 'pub def hello(who: Str) -> Str = "hello, #{who}"\n' > packages/greet/lib.lume
printf '[package]\nname = "other"\nversion = "0.1.0"\n' > packages/other/lume.toml
printf 'pub def x -> Int = 1\n' > packages/other/lib.lume
g add -A && g commit -qm one && g tag v1 && g clone -q --bare . "$tmp/mono.git"
mkdir -p "$tmp/usesmono"
printf '[package]\nname = "usesmono"\nversion = "0.1.0"\n\n[dependencies]\ngreet = { git = "../mono.git", tag = "v1" }\n' > "$tmp/usesmono/lume.toml"
printf 'import greet\n\ndef main:\n  puts greet.hello("lume")\n' > "$tmp/usesmono/main.lume"
run "$tmp/usesmono" run
printf '[package]\nname = "usesmono"\nversion = "0.1.0"\n\n[dependencies]\nmissing = { git = "../mono.git", tag = "v1" }\n' > "$tmp/usesmono/lume.toml"
run "$tmp/usesmono" check

step "a path dependency cannot be updated"
mkdir -p "$tmp/local"
printf '[package]\nname = "local"\nversion = "0.1.0"\n\n[dependencies]\ncolors = { path = "../src/colors" }\n' > "$tmp/local/lume.toml"
printf 'import colors\n\ndef main:\n  puts colors.red\n' > "$tmp/local/main.lume"
run "$tmp/local" update colors
run "$tmp/local" update

rm -rf "$tmp"
