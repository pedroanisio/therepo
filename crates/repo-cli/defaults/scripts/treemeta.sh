#!/usr/bin/env bash
set -euo pipefail

# ── defaults ──
DIR="."
FORMAT="tsv"           # tsv | json | jsonl | csv
DEPTH=""               # empty = unlimited
GITIGNORE=false
SHOW_CREATED=false
SHOW_LINES=false

# Auto-enable --gitignore inside git repos (node_modules, site, etc. are
# almost never wanted).  Use --no-gitignore to override.
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  GITIGNORE=true
fi

# ── filters ──
EXT=""                 # e.g. "ts,js,tsx"
NAME_PATTERN=""        # glob pattern, e.g. "*.test.*"
EXCLUDE_PATTERN=""     # glob to exclude, e.g. "*.min.*"
MODIFIED_AFTER=""      # ISO date: 2025-01-01
MODIFIED_BEFORE=""     # ISO date: 2025-12-31
MIN_LINES=""
MAX_LINES=""

# ── pagination ──
LIMIT=""               # max files to output
OFFSET=0               # skip first N files
PAGE=""                # shorthand: --page 3 --per-page 20
PER_PAGE=""

# ── sorting ──
SORT_BY="path"         # path | modified | created | lines
SORT_ORDER="asc"       # asc | desc

COUNT_ONLY=false

# ── usage ──
usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS] [DIR]

Show file metadata in token-friendly formats with filtering and pagination.

Output:
  -f, --format    FORMAT    tsv (default), json, jsonl, csv
  -c, --created             Include creation time
  -l, --lines               Include line count

Tree:
  -d, --depth     N         Max directory depth
  -g, --gitignore           Respect .gitignore (auto-enabled in git repos)
      --no-gitignore        Disable gitignore filtering (overrides auto-detect)

Filters:
  -e, --ext       EXT       Comma-separated extensions (e.g. ts,js,tsx)
  -n, --name      PATTERN   Glob pattern for filename (e.g. "*.test.*")
  -x, --exclude   PATTERN   Glob pattern to exclude (e.g. "*.min.*")
      --after     DATE      Modified after  (YYYY-MM-DD)
      --before    DATE      Modified before (YYYY-MM-DD)
      --min-lines N         Min line count (enables -l)
      --max-lines N         Max line count (enables -l)

Pagination:
      --limit     N         Max files to return
      --offset    N         Skip first N files
      --page      N         Page number (1-based, requires --per-page)
      --per-page  N         Files per page (requires --page)

Sorting:
  -s, --sort      FIELD     Sort by: path, modified, created, lines
  -o, --order     DIR       asc (default) or desc

Meta:
      --count               Print total matching file count and exit
  -h, --help                Show this help

Examples:
  $(basename "$0") -g -l -e ts,tsx --limit 50 ./src
  $(basename "$0") -f jsonl --after 2025-01-01 --sort modified -o desc .
  $(basename "$0") -g -l --page 2 --per-page 20 ./project
  $(basename "$0") --count -g -e py ./repo
  $(basename "$0") -g -l --min-lines 10 --max-lines 500 -x "*.min.*" .
EOF
  exit 0
}

# ── arg helpers ──
die() {
  echo "Error: $*" >&2
  exit 1
}

need_value() {
  [[ $# -ge 2 ]] || die "option '$1' requires a value"
}

is_nonneg_int() {
  [[ "$1" =~ ^[0-9]+$ ]]
}

is_pos_int() {
  [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

# ── parse args ──
while [[ $# -gt 0 ]]; do
  case "$1" in
    -f|--format)     need_value "$@"; FORMAT="$2";          shift 2 ;;
    -d|--depth)      need_value "$@"; DEPTH="$2";           shift 2 ;;
    -g|--gitignore)     GITIGNORE=true;       shift   ;;
    --no-gitignore)     GITIGNORE=false;      shift   ;;
    -c|--created)    SHOW_CREATED=true;    shift   ;;
    -l|--lines)      SHOW_LINES=true;      shift   ;;
    -e|--ext)        need_value "$@"; EXT="$2";             shift 2 ;;
    -n|--name)       need_value "$@"; NAME_PATTERN="$2";    shift 2 ;;
    -x|--exclude)    need_value "$@"; EXCLUDE_PATTERN="$2"; shift 2 ;;
    --after)         need_value "$@"; MODIFIED_AFTER="$2";  shift 2 ;;
    --before)        need_value "$@"; MODIFIED_BEFORE="$2"; shift 2 ;;
    --min-lines)     need_value "$@"; MIN_LINES="$2"; SHOW_LINES=true; shift 2 ;;
    --max-lines)     need_value "$@"; MAX_LINES="$2"; SHOW_LINES=true; shift 2 ;;
    --limit)         need_value "$@"; LIMIT="$2";           shift 2 ;;
    --offset)        need_value "$@"; OFFSET="$2";          shift 2 ;;
    --page)          need_value "$@"; PAGE="$2";            shift 2 ;;
    --per-page)      need_value "$@"; PER_PAGE="$2";        shift 2 ;;
    -s|--sort)       need_value "$@"; SORT_BY="$2";         shift 2 ;;
    -o|--order)      need_value "$@"; SORT_ORDER="$2";      shift 2 ;;
    --count)         COUNT_ONLY=true;      shift   ;;
    -h|--help)       usage                         ;;
    -*)              echo "Error: unknown option '$1'" >&2; exit 1 ;;
    *)               DIR="$1";             shift   ;;
  esac
done

# ── resolve page → limit/offset ──
if [[ -n "$DEPTH" ]]; then
  is_pos_int "$DEPTH" || die "--depth must be a positive integer"
fi
if [[ -n "$LIMIT" ]]; then
  is_nonneg_int "$LIMIT" || die "--limit must be a non-negative integer"
fi
is_nonneg_int "$OFFSET" || die "--offset must be a non-negative integer"
if [[ -n "$MIN_LINES" ]]; then
  is_nonneg_int "$MIN_LINES" || die "--min-lines must be a non-negative integer"
fi
if [[ -n "$MAX_LINES" ]]; then
  is_nonneg_int "$MAX_LINES" || die "--max-lines must be a non-negative integer"
fi
if [[ -n "$MIN_LINES" && -n "$MAX_LINES" && "$MIN_LINES" -gt "$MAX_LINES" ]]; then
  die "--min-lines cannot be greater than --max-lines"
fi

if [[ -n "$PAGE" || -n "$PER_PAGE" ]]; then
  [[ -n "$PAGE" && -n "$PER_PAGE" ]] || die "--page and --per-page must be used together"
  is_pos_int "$PAGE" || die "--page must be a positive integer"
  is_pos_int "$PER_PAGE" || die "--per-page must be a positive integer"
fi

if [[ -n "$PAGE" && -n "$PER_PAGE" ]]; then
  LIMIT="$PER_PAGE"
  OFFSET=$(( (PAGE - 1) * PER_PAGE ))
fi

# ── validate ──
[[ -d "$DIR" ]]                || { echo "Error: '$DIR' is not a directory" >&2; exit 1; }
command -v tree >/dev/null 2>&1 || { echo "Error: 'tree' is not installed" >&2; exit 1; }
case "$SORT_BY" in
  path|modified|created|lines) ;;
  *) die "unknown sort field '$SORT_BY' (expected: path, modified, created, lines)" ;;
esac
case "$SORT_ORDER" in
  asc|desc) ;;
  *) die "unknown sort order '$SORT_ORDER' (expected: asc or desc)" ;;
esac

# ── build tree command ──
TREE_ARGS=(-fi --noreport)
[[ -n "$DEPTH" ]]          && TREE_ARGS+=(-L "$DEPTH")
[[ "$GITIGNORE" == true ]] && TREE_ARGS+=(--gitignore)
TREE_ARGS+=("$DIR")

# ── separator ──
case "$FORMAT" in
  tsv)        SEP=$'\t' ;;
  csv)        SEP=","   ;;
  json|jsonl) SEP=""    ;;
  *)          echo "Error: unknown format '$FORMAT'" >&2; exit 1 ;;
esac

# ── build ext lookup ──
declare -A EXT_MAP=()
if [[ -n "$EXT" ]]; then
  IFS=',' read -ra EXTS <<< "$EXT"
  for e in "${EXTS[@]}"; do
    e="${e#.}"
    EXT_MAP["$e"]=1
  done
fi

# ── helpers ──
get_ext() {
  local b="${1##*/}"
  if [[ "$b" == *.* ]]; then echo "${b##*.}"; else echo ""; fi
}

date_to_epoch() {
  date -d "$1" +%s 2>/dev/null \
    || date -j -f "%Y-%m-%d" "$1" +%s 2>/dev/null \
    || echo 0
}

# Return "mtime_epoch<US>mtime_human" using a single stat call where possible.
stat_mtime_fields() {
  local out epoch human

  out=$(stat -c '%Y|%y' "$1" 2>/dev/null || true)
  if [[ -n "$out" ]]; then
    epoch="${out%%|*}"
    human="${out#*|}"
    human="${human%%.*}"
    printf '%s\x1f%s\n' "$epoch" "$human"
    return
  fi

  out=$(stat -f '%m|%Sm' -t '%Y-%m-%d %H:%M:%S' "$1" 2>/dev/null || true)
  if [[ -n "$out" ]]; then
    epoch="${out%%|*}"
    human="${out#*|}"
    printf '%s\x1f%s\n' "$epoch" "$human"
    return
  fi

  printf '0\x1f\n'
}

stat_mtime_human() {
  local out
  out=$(stat -c '%y' "$1" 2>/dev/null || true)
  if [[ -n "$out" ]]; then
    out="${out%%.*}"
    printf '%s\n' "$out"
    return
  fi

  stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$1" 2>/dev/null \
    || echo ""
}

stat_birth_human() {
  local c
  c=$(stat -c '%w' "$1" 2>/dev/null | cut -d. -f1 || true)
  if [[ -n "$c" && "$c" != "-" ]]; then
    echo "$c"
    return
  fi

  local b
  b=$(stat -f '%B' "$1" 2>/dev/null || echo 0)
  if [[ "$b" == "0" ]]; then
    echo "n/a"
  else
    date -r "$b" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "n/a"
  fi
}

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

csv_escape() {
  local s="$1"
  s="${s//\"/\"\"}"
  printf '"%s"' "$s"
}

# ── pre-compute date bounds ──
AFTER_EPOCH=0
BEFORE_EPOCH=99999999999
[[ -n "$MODIFIED_AFTER" ]]  && AFTER_EPOCH=$(date_to_epoch "$MODIFIED_AFTER")
[[ -n "$MODIFIED_BEFORE" ]] && BEFORE_EPOCH=$(date_to_epoch "$MODIFIED_BEFORE")
NEED_M_EPOCH=false
if [[ -n "$MODIFIED_AFTER" || -n "$MODIFIED_BEFORE" || "$SORT_BY" == "modified" ]]; then
  NEED_M_EPOCH=true
fi

# ── collect matching files into temp ──
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT
REC_SEP=$'\x1f'
HEADER_PRINTED=false

# Emit the header early for the common, non-paginated table outputs so users
# immediately see progress even on large trees.
if [[ "$FORMAT" != "jsonl" && "$FORMAT" != "json" && -z "$LIMIT" && "$OFFSET" -eq 0 && "$COUNT_ONLY" == false ]]; then
  h="path${SEP}modified"
  [[ "$SHOW_CREATED" == true ]] && h+="${SEP}created"
  [[ "$SHOW_LINES"   == true ]] && h+="${SEP}lines"
  echo "$h"
  HEADER_PRINTED=true
fi

tree "${TREE_ARGS[@]}" 2>/dev/null | while IFS= read -r f; do
  [[ -f "$f" ]] || continue

  # extension filter
  if [[ ${#EXT_MAP[@]} -gt 0 ]]; then
    fext=$(get_ext "$f")
    [[ -z "$fext" || -z "${EXT_MAP[$fext]+x}" ]] && continue
  fi

  # name pattern filter
  if [[ -n "$NAME_PATTERN" ]]; then
    fname="${f##*/}"
    case "$fname" in $NAME_PATTERN) ;; *) continue ;; esac
  fi

  # exclude pattern filter
  if [[ -n "$EXCLUDE_PATTERN" ]]; then
    fname="${f##*/}"
    case "$fname" in $EXCLUDE_PATTERN) continue ;; esac
  fi

  # modified time
  m_epoch=0
  if [[ "$NEED_M_EPOCH" == true ]]; then
    IFS="$REC_SEP" read -r m_epoch m < <(stat_mtime_fields "$f")
  else
    m=$(stat_mtime_human "$f")
  fi

  # date range filter
  if [[ "$NEED_M_EPOCH" == true ]]; then
    [[ "$m_epoch" -lt "$AFTER_EPOCH" ]]  && continue
    [[ "$m_epoch" -gt "$BEFORE_EPOCH" ]] && continue
  fi

  # created time
  c=""
  if [[ "$SHOW_CREATED" == true ]]; then
    c=$(stat_birth_human "$f")
  fi

  # line count
  lc=0
  if [[ "$SHOW_LINES" == true ]]; then
    lc=$(wc -l < "$f" 2>/dev/null || echo 0)
    lc="${lc// /}"
    [[ -n "$MIN_LINES" && "$lc" -lt "$MIN_LINES" ]] && continue
    [[ -n "$MAX_LINES" && "$lc" -gt "$MAX_LINES" ]] && continue
  fi

  # write record: path<US>modified<US>m_epoch<US>created<US>lines
  printf '%s\x1f%s\x1f%s\x1f%s\x1f%s\n' "$f" "$m" "$m_epoch" "$c" "$lc"
done > "$TMPFILE"

# ── count mode ──
if [[ "$COUNT_ONLY" == true ]]; then
  wc -l < "$TMPFILE" | tr -d ' '
  exit 0
fi

# ── sort ──
SORT_COL=1
case "$SORT_BY" in
  path)     SORT_COL=1 ;;
  modified) SORT_COL=3 ;;
  created)  SORT_COL=4 ;;
  lines)    SORT_COL=5 ;;
esac

SORT_FLAGS=("-t$REC_SEP" "-k${SORT_COL},${SORT_COL}")
[[ "$SORT_ORDER" == "desc" ]] && SORT_FLAGS+=("-r")
[[ "$SORT_BY" == "modified" || "$SORT_BY" == "lines" ]] && SORT_FLAGS+=("-n")

SORTED_FILE=$(mktemp)
trap 'rm -f "$TMPFILE" "$SORTED_FILE"' EXIT
sort "${SORT_FLAGS[@]}" "$TMPFILE" > "$SORTED_FILE"

# ── pagination ──
TOTAL=$(wc -l < "$SORTED_FILE" | tr -d ' ')

PAGINATED_FILE=$(mktemp)
trap 'rm -f "$TMPFILE" "$SORTED_FILE" "$PAGINATED_FILE"' EXIT

tail -n +"$((OFFSET + 1))" "$SORTED_FILE" > "$PAGINATED_FILE"
if [[ -n "$LIMIT" ]]; then
  head -n "$LIMIT" "$PAGINATED_FILE" > "${PAGINATED_FILE}.tmp"
  mv "${PAGINATED_FILE}.tmp" "$PAGINATED_FILE"
fi

SHOWN=$(wc -l < "$PAGINATED_FILE" | tr -d ' ')

# ── output: meta ──
if [[ -n "$LIMIT" || "$OFFSET" -gt 0 ]]; then
  case "$FORMAT" in
    json)  ;; # meta is embedded in the JSON wrapper below
    jsonl) echo "{\"_meta\":{\"total\":${TOTAL},\"offset\":${OFFSET},\"shown\":${SHOWN}}}" ;;
    *)
      echo "_total${SEP}_offset${SEP}_shown"
      echo "${TOTAL}${SEP}${OFFSET}${SEP}${SHOWN}"
      echo ""
      ;;
  esac
fi

# ── output: header ──
if [[ "$FORMAT" != "jsonl" && "$FORMAT" != "json" && "$HEADER_PRINTED" != true ]]; then
  h="path${SEP}modified"
  [[ "$SHOW_CREATED" == true ]] && h+="${SEP}created"
  [[ "$SHOW_LINES"   == true ]] && h+="${SEP}lines"
  echo "$h"
fi

# ── output: rows ──

# json format: build a complete JSON object with optional _meta and files array
if [[ "$FORMAT" == "json" ]]; then
  printf '{'
  if [[ -n "$LIMIT" || "$OFFSET" -gt 0 ]]; then
    printf '"_meta":{"total":%d,"offset":%d,"shown":%d},' "$TOTAL" "$OFFSET" "$SHOWN"
  fi
  printf '"files":['
  FIRST=true
  while IFS="$REC_SEP" read -r p m _me c lc; do
    [[ -z "$p" ]] && continue
    [[ "$FIRST" == true ]] && FIRST=false || printf ','
    json="{\"p\":\"$(json_escape "$p")\",\"m\":\"$(json_escape "$m")\""
    [[ "$SHOW_CREATED" == true ]] && json+=",\"c\":\"$(json_escape "$c")\""
    [[ "$SHOW_LINES"   == true ]] && json+=",\"l\":${lc}"
    json+="}"
    printf '%s' "$json"
  done < "$PAGINATED_FILE"
  printf ']}\n'
else
  while IFS="$REC_SEP" read -r p m _me c lc; do
    [[ -z "$p" ]] && continue
    case "$FORMAT" in
      tsv)
        row="${p}${SEP}${m}"
        [[ "$SHOW_CREATED" == true ]] && row+="${SEP}${c}"
        [[ "$SHOW_LINES"   == true ]] && row+="${SEP}${lc}"
        echo "$row"
        ;;
      csv)
        row="$(csv_escape "$p")${SEP}$(csv_escape "$m")"
        [[ "$SHOW_CREATED" == true ]] && row+="${SEP}$(csv_escape "$c")"
        [[ "$SHOW_LINES"   == true ]] && row+="${SEP}${lc}"
        echo "$row"
        ;;
      jsonl)
        json="{\"p\":\"$(json_escape "$p")\",\"m\":\"$(json_escape "$m")\""
        [[ "$SHOW_CREATED" == true ]] && json+=",\"c\":\"$(json_escape "$c")\""
        [[ "$SHOW_LINES"   == true ]] && json+=",\"l\":${lc}"
        json+="}"
        echo "$json"
        ;;
    esac
  done < "$PAGINATED_FILE"
fi
