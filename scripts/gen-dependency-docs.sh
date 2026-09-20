#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

# Regenerates docs/dependency-docs: offline rustdoc HTML for drtm's
# dependencies. The index page is styled like rustdoc's own pages by
# reusing the <head> of a freshly generated page, since its CSS/JS
# filenames are content-hashed per rustdoc version and can't be hardcoded.
# Each crate's root page also gets a link back to that index spliced in.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

crates=(log uefi uefi_raw)

cargo doc --no-deps -p log -p uefi -p uefi-raw

doc_dir="target/x86_64-unknown-uefi/doc"
out_dir="docs/dependency-docs"
rustdoc_dir="$out_dir/rustdoc"

rm -rf "$out_dir"
mkdir -p "$out_dir"
cp -r "$doc_dir" "$rustdoc_dir"

# The source page lives two directories below docs/dependency-docs
# (rustdoc/<crate>/index.html), so its asset paths are all "../foo";
# our index.html lives at that root and needs "rustdoc/foo" instead.
head_html="$(grep -oP '<head>.*?</head>' "$rustdoc_dir/${crates[0]}/index.html" \
  | sed 's#\.\./#rustdoc/#g')"

{
  echo "<!DOCTYPE html><html lang=\"en\">"
  echo "$head_html"
  echo '<body class="rustdoc mod sys"><main><div class="width-limiter">'
  echo '<section id="main-content" class="content" tabindex="-1">'
  echo '<div class="main-heading"><h1>drtm dependency docs</h1></div>'
  echo '<ul class="all-items">'
  for c in "${crates[@]}"; do
    echo "<li><a href=\"rustdoc/${c}/index.html\">${c}</a></li>"
  done
  echo '</ul>'
  echo '</section></div></main></body></html>'
} >"$out_dir/index.html"

for c in "${crates[@]}"; do
  sed -i 's#<div class="main-heading">#<div class="main-heading">'\
'<p class="rustdoc-back-link"><a href="../../index.html">« drtm dependency docs</a></p>#' \
    "$rustdoc_dir/$c/index.html"
done
