#!/bin/sh
set -eu

FORGEJO_URL="${SKWD_FORGEJO_URL:-}"
RELEASE_REPOSITORY="${SKWD_RELEASE_REPOSITORY:-liixini/skwd-release}"
RELEASE_KEYRING="${SKWD_RELEASE_KEYRING:-}"
RELEASE_KEY_FINGERPRINT="${SKWD_RELEASE_KEY_FINGERPRINT:-}"
BINDIR="${SKWD_BINDIR:-$HOME/.local/bin}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
INSTALLER_CONTRACT=skwd-portable-suite-v1
INSTALLER_MAX_VALIDITY=2678400
INSTALLER_CLOCK_SKEW=300

bold() { printf '\033[1m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v date >/dev/null 2>&1 || die "date is required"
command -v gpgv >/dev/null 2>&1 || die "gpgv is required"
command -v tar >/dev/null 2>&1 || die "tar is required"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required"
FORGEJO_URL=${FORGEJO_URL%/}
case "$FORGEJO_URL" in
    http://*|https://*) ;;
    *) die "SKWD_FORGEJO_URL must be an HTTP or HTTPS origin" ;;
esac
printf '%s\n' "$RELEASE_REPOSITORY" \
    | grep -Eq '^[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*$' \
    || die "SKWD_RELEASE_REPOSITORY must be a safe OWNER/REPOSITORY pair"
RELEASE_OWNER=${RELEASE_REPOSITORY%%/*}
forgejo_authority=${FORGEJO_URL#*://}
case "$forgejo_authority" in
    ""|*/*|*\?*|*\#*|*@*) die "SKWD_FORGEJO_URL must contain only a scheme and authority" ;;
esac
if [ -n "${SKWD_FORGEJO_TOKEN:-}" ]; then
    case "$FORGEJO_URL" in
        https://*) ;;
        *) die "refusing to send SKWD_FORGEJO_TOKEN over non-HTTPS Forgejo" ;;
    esac
    case "$SKWD_FORGEJO_TOKEN" in
        *[!A-Za-z0-9._~-]*) die "SKWD_FORGEJO_TOKEN contains unsupported characters" ;;
    esac
fi
printf '%s\n' "$RELEASE_KEY_FINGERPRINT" \
    | grep -Eq '^[0-9A-Fa-f]{40}([0-9A-Fa-f]{24})?$' \
    || die "SKWD_RELEASE_KEY_FINGERPRINT must be the independently verified publisher fingerprint"
RELEASE_KEY_FINGERPRINT=$(printf '%s' "$RELEASE_KEY_FINGERPRINT" \
    | tr '[:lower:]' '[:upper:]')
if [ -n "$RELEASE_KEYRING" ]; then
    [ -f "$RELEASE_KEYRING" ] && [ ! -L "$RELEASE_KEYRING" ] \
        || die "release keyring is not a regular file: $RELEASE_KEYRING"
fi

fetch_json() {
    case "$1" in
        "$FORGEJO_URL"/*) ;;
        *) die "refusing a release URL outside the configured Forgejo origin: $1" ;;
    esac
    if [ -n "${SKWD_FORGEJO_TOKEN:-}" ]; then
        curl --disable -fsSL --max-redirs=0 --proto='=http,https' \
            --config "$curl_auth" "$1"
    else
        curl --disable -fsSL --max-redirs=5 --proto='=http,https' --proto-redir='=http,https' "$1"
    fi
}

download() {
    case "$1" in
        "$FORGEJO_URL"/*) ;;
        *) die "refusing a release URL outside the configured Forgejo origin: $1" ;;
    esac
    if [ -n "${SKWD_FORGEJO_TOKEN:-}" ]; then
        curl --disable -fL --max-redirs=0 --proto='=http,https' \
            --config "$curl_auth" "$1" -o "$2"
    else
        curl --disable -fL --max-redirs=5 --proto='=http,https' --proto-redir='=http,https' \
            "$1" -o "$2"
    fi
}

verify_manifest_signature() {
    keyring=$1
    signature=$2
    manifest=$3
    status=$4
    gpgv --status-fd 1 --keyring "$keyring" "$signature" "$manifest" \
        > "$status" 2>/dev/null \
        || die "coordinated release signature verification failed"
    awk -v expected="$RELEASE_KEY_FINGERPRINT" '
        $1 == "[GNUPG:]" && $2 == "VALIDSIG" {
            signed = toupper($3)
            primary = toupper($12)
            if (signed == expected || primary == expected) valid = 1
        }
        END { exit !valid }
    ' "$status" || die "coordinated release was not signed by the reviewed publisher key"
}

validate_manifest_shape() {
    awk -F '\t' -v contract="$INSTALLER_CONTRACT" -v arch="$ARCH" \
        -v release_repository="$RELEASE_REPOSITORY" -v release_owner="$RELEASE_OWNER" '
        function hex(value, size) {
            return length(value) == size && value !~ /[^0-9A-Fa-f]/
        }
        NR == 1 { valid = ($0 == "SKWD-INSTALL-MANIFEST\t1"); next }
        NR == 2 { valid = valid && $1 == "contract" && $2 == contract && NF == 2; next }
        NR == 3 { valid = valid && $1 == "architecture" && $2 == arch && NF == 2; next }
        NR == 4 {
            version = $2
            valid = valid && $1 == "version" && NF == 2 \
                && version ~ /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/
            next
        }
        NR == 5 { valid = valid && $1 == "tag" && $2 == "v" version && NF == 2; next }
        NR == 6 { valid = valid && $1 == "issued-at" && $2 ~ /^[0-9]+$/ && NF == 2; next }
        NR == 7 { valid = valid && $1 == "expires-at" && $2 ~ /^[0-9]+$/ && NF == 2; next }
        NR == 8 {
            valid = valid && $1 == "release" && $2 == release_repository \
                && hex($3, 40) && NF == 3
            next
        }
        NR >= 9 && NR <= 12 {
            split("wall deck paper lens", ordered, " ")
            component = ordered[NR - 8]
            expected_repository = release_owner "/skwd-" component
            expected_archive = "skwd-" component "_" version "_linux-" arch ".tar.xz"
            valid = valid && $1 == "component" && $2 == component \
                && $3 == expected_repository && hex($4, 40) \
                && $5 == expected_archive && hex($6, 64) \
                && $7 ~ /^[1-9][0-9]*$/ && NF == 7
            next
        }
        { valid = 0 }
        END { exit !(valid && NR == 12) }
    ' "$1" || die "signed installer manifest has an invalid coordinated interface"
}

manifest_field() {
    awk -F '\t' -v key="$1" '$1 == key { print $2 }' "$2"
}

manifest_component_field() {
    awk -F '\t' -v component="$1" -v field="$2" \
        '$1 == "component" && $2 == component { print $field }' "$3"
}

version_order() {
    awk -v previous="$1" -v candidate="$2" '
        BEGIN {
            split(previous, old, ".")
            split(candidate, new, ".")
            for (part = 1; part <= 3; part++) {
                if (length(new[part]) < length(old[part])) { print -1; exit }
                if (length(new[part]) > length(old[part])) { print 1; exit }
                if (("x" new[part]) < ("x" old[part])) { print -1; exit }
                if (("x" new[part]) > ("x" old[part])) { print 1; exit }
            }
            print 0
        }
    '
}

case "$(uname -m)" in
    x86_64) ARCH=x86_64 ;;
    *) die "unsupported architecture: $(uname -m); Skwd release binaries currently support Linux x86_64 only" ;;
esac

temporary=$(mktemp -d)
temporary_parent=${TMPDIR:-/tmp}
temporary_parent=${temporary_parent%/}
transaction_plan="$temporary/install-plan"
transaction_applied="$temporary/install-applied"
transaction_started=0
transaction_committed=0
curl_auth=
if [ -n "${SKWD_FORGEJO_TOKEN:-}" ]; then
    curl_auth="$temporary/curl-auth"
    (umask 077; printf 'header = "Authorization: token %s"\n' \
        "$SKWD_FORGEJO_TOKEN" > "$curl_auth")
fi

transaction_path() {
    destination=$1
    suffix=$2
    printf '%s.skwd-%s-%s\n' "$destination" "$suffix" "$$"
}

remove_transaction_path() {
    path=$1
    case "$path" in
        *.skwd-new-"$$"|*.skwd-old-"$$"|*.skwd-failed-"$$")
            [ ! -e "$path" ] && [ ! -L "$path" ] || rm -rf -- "$path"
            ;;
        *) warn "refusing to remove unexpected transaction path: $path" ;;
    esac
}

rollback_install() {
    [ "$transaction_started" -eq 1 ] || return 0
    [ "$transaction_committed" -eq 0 ] || return 0
    [ -f "$transaction_applied" ] || return 0
    awk '{ lines[NR] = $0 } END { for (line = NR; line >= 1; line--) print lines[line] }' \
        "$transaction_applied" | while IFS= read -r destination; do
        [ -n "$destination" ] || continue
        backup=$(transaction_path "$destination" old)
        failed=$(transaction_path "$destination" failed)
        if [ -e "$destination" ] || [ -L "$destination" ]; then
            mv -- "$destination" "$failed" || true
        fi
        if [ -e "$backup" ] || [ -L "$backup" ]; then
            mv -- "$backup" "$destination" || true
        fi
        remove_transaction_path "$failed"
    done
}

cleanup() {
    rollback_install
    if [ -f "$transaction_plan" ]; then
        while IFS= read -r destination; do
            [ -n "$destination" ] || continue
            remove_transaction_path "$(transaction_path "$destination" new)"
            if [ "$transaction_committed" -eq 1 ]; then
                remove_transaction_path "$(transaction_path "$destination" old)"
            fi
        done < "$transaction_plan"
    fi
    [ -n "${temporary:-}" ] || return 0
    case "$temporary" in
        "$temporary_parent"/tmp.*) ;;
        *) warn "refusing to remove unexpected temporary path: $temporary"; return 0 ;;
    esac
    [ ! -L "$temporary" ] || {
        warn "refusing to remove a replaced temporary directory: $temporary"
        return 0
    }
    [ ! -e "$temporary" ] || rm -rf -- "$temporary"
}
trap cleanup EXIT HUP INT TERM

plan_destination() {
    destination=$1
    case "$destination" in
        ""|*'
'*) die "install destination contains an unsupported newline" ;;
    esac
    if [ -f "$transaction_plan" ] && grep -Fqx -- "$destination" "$transaction_plan"; then
        die "duplicate install destination: $destination"
    fi
    parent=$(dirname -- "$destination")
    mkdir -p "$parent"
    prepared=$(transaction_path "$destination" new)
    backup=$(transaction_path "$destination" old)
    [ ! -e "$prepared" ] && [ ! -L "$prepared" ] \
        || die "temporary install destination already exists: $prepared"
    [ ! -e "$backup" ] && [ ! -L "$backup" ] \
        || die "backup install destination already exists: $backup"
    printf '%s\n' "$destination" >> "$transaction_plan"
}

prepare_file() {
    source=$1
    destination=$2
    mode=$3
    plan_destination "$destination"
    install -m"$mode" "$source" "$(transaction_path "$destination" new)"
}

prepare_tree() {
    source=$1
    destination=$2
    plan_destination "$destination"
    prepared=$(transaction_path "$destination" new)
    mkdir "$prepared"
    cp -a "$source/." "$prepared/"
}

commit_install() {
    transaction_started=1
    : > "$transaction_applied"
    while IFS= read -r destination; do
        [ -n "$destination" ] || continue
        prepared=$(transaction_path "$destination" new)
        backup=$(transaction_path "$destination" old)
        if [ -e "$destination" ] || [ -L "$destination" ]; then
            mv -- "$destination" "$backup"
        fi
        printf '%s\n' "$destination" >> "$transaction_applied"
        mv -- "$prepared" "$destination"
    done < "$transaction_plan"
    transaction_committed=1
}

release_json="$temporary/suite-release.json"
release_manifest="$temporary/SKWD-INSTALL-MANIFEST"
release_signature="$temporary/SKWD-INSTALL-MANIFEST.asc"
downloaded_keyring="$temporary/skwd-release-keyring.gpg"
signature_status="$temporary/signature-status"
bold "finding the latest coordinated Skwd release"
fetch_json "$FORGEJO_URL/api/v1/repos/$RELEASE_REPOSITORY/releases/latest" > "$release_json"
release_manifest_url=$(grep -oE '"browser_download_url":[[:space:]]*"[^"]*/SKWD-INSTALL-MANIFEST"' "$release_json" \
    | head -1 | sed 's/.*"\(https\{0,1\}:[^"[:space:]]*\)"/\1/')
release_signature_url=$(grep -oE '"browser_download_url":[[:space:]]*"[^"]*/SKWD-INSTALL-MANIFEST\.asc"' "$release_json" \
    | head -1 | sed 's/.*"\(https\{0,1\}:[^"[:space:]]*\)"/\1/')
release_keyring_url=$(grep -oE '"browser_download_url":[[:space:]]*"[^"]*/skwd-release-keyring\.gpg"' "$release_json" \
    | head -1 | sed 's/.*"\(https\{0,1\}:[^"[:space:]]*\)"/\1/')
[ -n "$release_manifest_url" ] || die "coordinated release does not publish SKWD-INSTALL-MANIFEST"
[ -n "$release_signature_url" ] || die "coordinated release does not publish SKWD-INSTALL-MANIFEST.asc"
download "$release_manifest_url" "$release_manifest"
download "$release_signature_url" "$release_signature"
if [ -z "$RELEASE_KEYRING" ]; then
    [ -n "$release_keyring_url" ] \
        || die "coordinated release does not publish skwd-release-keyring.gpg"
    download "$release_keyring_url" "$downloaded_keyring"
    RELEASE_KEYRING=$downloaded_keyring
fi
verify_manifest_signature \
    "$RELEASE_KEYRING" "$release_signature" "$release_manifest" "$signature_status"
validate_manifest_shape "$release_manifest"

release_version=$(manifest_field version "$release_manifest")
release_tag=$(manifest_field tag "$release_manifest")
issued_at=$(manifest_field issued-at "$release_manifest")
expires_at=$(manifest_field expires-at "$release_manifest")
now=$(date +%s)
printf '%s\n' "$now" | grep -Eq '^[0-9]+$' || die "date returned an invalid epoch"
[ "$issued_at" -le $((now + INSTALLER_CLOCK_SKEW)) ] \
    || die "signed release manifest is not valid yet"
[ "$expires_at" -gt "$now" ] || die "signed release manifest has expired"
[ "$expires_at" -gt "$issued_at" ] \
    && [ $((expires_at - issued_at)) -le "$INSTALLER_MAX_VALIDITY" ] \
    || die "signed release manifest has an excessive validity window"
release_api_tag=$(grep -oE '"tag_name":[[:space:]]*"v[0-9]+\.[0-9]+\.[0-9]+"' "$release_json" \
    | head -1 | sed 's/.*"\([^"]*\)"/\1/')
[ "$release_api_tag" = "$release_tag" ] \
    || die "latest coordinated release tag does not match its signed manifest"

installed_manifest="$STATE_HOME/skwd-wall-v2/portable-release.manifest"
installed_signature="$STATE_HOME/skwd-wall-v2/portable-release.manifest.asc"
if [ -e "$installed_manifest" ] || [ -e "$installed_signature" ]; then
    [ -f "$installed_manifest" ] && [ ! -L "$installed_manifest" ] \
        && [ -f "$installed_signature" ] && [ ! -L "$installed_signature" ] \
        || die "installed portable release state is incomplete or unsafe"
    previous_status="$temporary/previous-signature-status"
    verify_manifest_signature \
        "$RELEASE_KEYRING" "$installed_signature" "$installed_manifest" "$previous_status"
    validate_manifest_shape "$installed_manifest"
    previous_version=$(manifest_field version "$installed_manifest")
    order=$(version_order "$previous_version" "$release_version")
    [ "$order" -ge 0 ] \
        || die "refusing signed release rollback from $previous_version to $release_version"
    if [ "$order" -eq 0 ]; then
        previous_digest=$(sha256sum "$installed_manifest" | awk '{print $1}')
        candidate_digest=$(sha256sum "$release_manifest" | awk '{print $1}')
        [ "$previous_digest" = "$candidate_digest" ] \
            || die "signed release reuses version $release_version with different content"
    fi
fi

fetch_component() {
    component=$1
    repository=$(manifest_component_field "$component" 3 "$release_manifest")
    archive_name=$(manifest_component_field "$component" 5 "$release_manifest")
    expected=$(manifest_component_field "$component" 6 "$release_manifest")
    expected_size=$(manifest_component_field "$component" 7 "$release_manifest")
    bold "finding signed $component asset $archive_name"
    release_json="$temporary/$component-release.json"
    fetch_json "$FORGEJO_URL/api/v1/repos/$repository/releases/tags/$release_tag" > "$release_json"
    component_api_tag=$(grep -oE '"tag_name":[[:space:]]*"v[0-9]+\.[0-9]+\.[0-9]+"' "$release_json" \
        | head -1 | sed 's/.*"\([^"]*\)"/\1/')
    [ "$component_api_tag" = "$release_tag" ] \
        || die "$component release tag does not match the signed manifest"
    asset=$(grep -oE '"browser_download_url":[[:space:]]*"[^"]*/'"$archive_name"'"' "$release_json" \
        | head -1 | sed 's/.*"\(https\{0,1\}:[^"[:space:]]*\)"/\1/')
    [ -n "$asset" ] || die "signed $component asset is not published: $archive_name"
    archive="$temporary/$component-$archive_name"
    download "$asset" "$archive"
    printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --status \
        || die "SHA-256 verification failed for $archive_name"
    actual_size=$(wc -c < "$archive" | tr -d '[:space:]')
    [ "$actual_size" = "$expected_size" ] || die "size verification failed for $archive_name"
    if ! tar -tf "$archive" | awk '
        /^\// || /(^|\/)\.\.($|\/)/ || !/^usr(\/|$)/ || seen[$0]++ { bad = 1 }
        END { exit bad }
    '; then
        die "$component archive contains an unsafe path"
    fi
    if tar -tvf "$archive" | awk 'substr($1, 1, 1) != "-" && substr($1, 1, 1) != "d" { bad = 1 } END { exit bad }'; then
        :
    else
        die "$component archive contains a non-regular entry"
    fi
    component_root="$temporary/$component"
    mkdir "$component_root"
    tar --no-same-owner --no-same-permissions -C "$component_root" -xf "$archive"
    [ -z "$(find "$component_root" -type l -print -quit)" ] \
        || die "$component archive contains a symbolic link"
}

fetch_component wall
fetch_component deck
fetch_component paper
fetch_component lens

[ -x "$temporary/wall/usr/bin/skwd-wall-v2" ] || die "unexpected Wall archive layout"
for binary in skwd-walld skwd-wall-scan skwd-wall-effects skwd-wall-draw skwd-helm; do
    [ -x "$temporary/deck/usr/bin/$binary" ] || die "Deck archive is missing $binary"
done
[ -f "$temporary/deck/usr/lib/systemd/user/skwd-walld.service" ] \
    || die "Deck archive is missing skwd-walld.service"
for binary in skwd-paper-v2 skwd-wall-still skwd-wall-vk; do
    [ -x "$temporary/paper/usr/bin/$binary" ] || die "Paper archive is missing $binary"
done
[ -x "$temporary/lens/usr/bin/skwd-lens" ] || die "Lens archive is missing skwd-lens"

for component in wall deck paper lens; do
    case "$component" in
        wall) license_source="$temporary/wall/usr/share/licenses/skwd-wall-v2" ;;
        *) license_source="$temporary/$component/usr/share/licenses/skwd-$component" ;;
    esac
    [ -f "$license_source/LICENSE" ] && [ ! -L "$license_source/LICENSE" ] \
        || die "$component archive is missing its product license"
    [ -d "$license_source/third-party" ] && [ ! -L "$license_source/third-party" ] \
        || die "$component archive is missing its third-party license inventory"
    [ -n "$(find "$license_source/third-party" -type f -print -quit)" ] \
        || die "$component archive has an empty third-party license inventory"
done

for binary in \
    skwd-wall-v2 \
    skwd-walld \
    skwd-wall-scan \
    skwd-wall-effects \
    skwd-wall-draw \
    skwd-helm \
    skwd-paper-v2 \
    skwd-wall-still \
    skwd-wall-vk \
    skwd-lens
do
    case "$binary" in
        skwd-wall-v2) source="$temporary/wall/usr/bin/$binary" ;;
        skwd-walld|skwd-wall-scan|skwd-wall-effects|skwd-wall-draw|skwd-helm)
            source="$temporary/deck/usr/bin/$binary"
            ;;
        skwd-paper-v2|skwd-wall-still|skwd-wall-vk)
            source="$temporary/paper/usr/bin/$binary"
            ;;
        skwd-lens) source="$temporary/lens/usr/bin/$binary" ;;
    esac
    prepare_file "$source" "$BINDIR/$binary" 755
done

desktop="$temporary/wall/usr/share/applications/skwd-wall-v2.desktop"
if [ -f "$desktop" ]; then
    prepare_file "$desktop" "$DATA_HOME/applications/skwd-wall-v2.desktop" 644
fi

icon="$temporary/wall/usr/share/icons/hicolor/scalable/apps/skwd-wall-v2.svg"
if [ -f "$icon" ]; then
    prepare_file "$icon" "$DATA_HOME/icons/hicolor/scalable/apps/skwd-wall-v2.svg" 644
fi

metainfo="$temporary/wall/usr/share/metainfo/org.skwd.wall.v2.metainfo.xml"
if [ -f "$metainfo" ]; then
    prepare_file "$metainfo" "$DATA_HOME/metainfo/org.skwd.wall.v2.metainfo.xml" 644
fi

for component in wall deck paper lens; do
    case "$component" in
        wall)
            license_source="$temporary/wall/usr/share/licenses/skwd-wall-v2"
            license_destination="$DATA_HOME/licenses/skwd-wall-v2"
            ;;
        *)
            license_source="$temporary/$component/usr/share/licenses/skwd-$component"
            license_destination="$DATA_HOME/licenses/skwd-$component"
            ;;
    esac
    prepare_tree "$license_source" "$license_destination"
done

templates="$temporary/deck/usr/share/skwd-wall-v2/data/matugen/templates"
if [ -d "$templates" ]; then
    prepare_tree "$templates" "$DATA_HOME/skwd-wall-v2/data/matugen/templates"
fi

service_source="$temporary/deck/usr/lib/systemd/user/skwd-walld.service"
service_installed=0
if [ "$BINDIR" = "$HOME/.local/bin" ] && [ -f "$service_source" ]; then
    service_directory="$CONFIG_HOME/systemd/user"
    sed 's|^ExecStart=/usr/bin/skwd-walld|ExecStart=%h/.local/bin/skwd-walld|' \
        "$service_source" > "$temporary/skwd-walld.service"
    prepare_file "$temporary/skwd-walld.service" "$service_directory/skwd-walld.service" 644
    service_installed=1
fi

state_directory="$STATE_HOME/skwd-wall-v2"
prepare_file "$release_manifest" "$state_directory/portable-release.manifest" 644
prepare_file "$release_signature" "$state_directory/portable-release.manifest.asc" 644
prepare_file "$RELEASE_KEYRING" "$state_directory/portable-release-keyring.gpg" 644

commit_install
bold "installed the Wall, Deck, Paper, and Lens executables to $BINDIR"
bold "installed coordinated license material to $DATA_HOME/licenses"
bold "recorded signed release $release_version for rollback protection"

if [ "$service_installed" -eq 1 ]; then
    if command -v systemctl >/dev/null 2>&1; then
        if systemctl --user daemon-reload \
            && systemctl --user enable --now skwd-walld.service
        then
            bold "enabled and started skwd-walld.service"
        else
            warn "installed the user service but could not start it in this session"
        fi
    fi
else
    warn "custom SKWD_BINDIR selected; start '$BINDIR/skwd-walld --wait-for-session' from your session"
fi

case ":$PATH:" in
    *":$BINDIR:"*) ;;
    *) warn "$BINDIR is not on PATH; add: export PATH=\"$BINDIR:\$PATH\"" ;;
esac

bold "done; launch the picker with skwd-wall-v2; Lens model packs remain separate"
